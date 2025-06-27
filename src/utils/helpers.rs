use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::extract::multipart::Field;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::{StreamExt, TryStreamExt};
use image::ImageReader;
use rand::Rng;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{error, info};

use crate::error_handling::app_error::AppError;

use redis::AsyncCommands;

pub async fn publish_message<T: Serialize>(
    redis: &Pool<RedisConnectionManager>,
    channel: &str,
    message: T,
) -> Result<(), AppError> {
    let mut redis_conn = redis
        .get()
        .await
        .map_err(|_| AppError::Internal("Failed to get Redis connection".to_string()))?;
    let serialized_message = serde_json::to_string(&message)
        .map_err(|_| AppError::Internal("Failed to serialize message".to_string()))?;

    redis_conn
        .publish::<_, _, ()>(channel, serialized_message)
        .await
        .map_err(|e| {
            tracing::error!("Failed to publish message to Redis: {}", e);
            AppError::Internal("Failed to publish message to Redis".to_string())
        })?;
    Ok(())
}

pub async fn get_cache_data<T: DeserializeOwned>(
    redis: &Pool<RedisConnectionManager>,
    cache_key: &str,
) -> Result<Option<T>, AppError> {
    let mut redis_conn = redis.get().await?;
    let cache_data: Option<String> = redis_conn.get(cache_key).await?;
    if let Some(cache_data) = cache_data {
        if let Ok(parsed) = serde_json::from_str::<T>(&cache_data) {
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

pub async fn delete_cache_data(
    redis: &Pool<RedisConnectionManager>,
    cache_key: &str,
) -> Result<(), AppError> {
    let mut redis_conn = redis.get().await?;
    redis_conn.del::<_, ()>(cache_key).await?;
    Ok(())
}

pub async fn set_cache_data<T: serde::Serialize>(
    redis: &Pool<RedisConnectionManager>,
    cache_key: &str,
    data: &T,
    expire_secs: u64,
) -> Result<(), AppError> {
    let mut redis_conn = redis.get().await?;
    let serialized = serde_json::to_string(&data)?;

    redis_conn
        .set_ex::<_, _, ()>(cache_key, serialized, expire_secs)
        .await?;
    Ok(())
}

pub async fn resize_image_from_bytes(original: Vec<u8>) -> Result<Vec<u8>, AppError> {
    // Decode image from buffer
    let img = ImageReader::new(std::io::Cursor::new(original))
        .with_guessed_format()
        .map_err(|e| AppError::Internal(format!("Image format error: {e}")))?
        .decode()
        .map_err(|e| AppError::Internal(format!("Decode image error: {e}")))?;

    // Resize to 1920 x 1080
    let resized = img.resize(1920, 1080, image::imageops::FilterType::Triangle);

    // return to jpeg
    let mut out = Vec::new();
    resized
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::Jpeg,
        )
        .map_err(|e| AppError::Internal(format!("Encode image error: {e}")))?;

    Ok(out)
}

pub async fn read_bytes_from_multipart_field<'a>(
    field: Field<'a>,
    max_size: usize,
) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    let mut total_size = 0usize;

    let mut stream = field.into_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| {
            AppError::BadRequest("Failed to read chunk, make sure your file is under 10 MB size and 1920x1080 resolution if it's an image".to_string())
        })?;

        total_size += chunk.len();
        if total_size > max_size {
            return Err(AppError::BadRequest("File too large".to_string()));
        }

        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hashed).map_err(|e| {
        error!("Password hash parse failed: {:?}", e);
        AppError::AuthError(format!("Username or password is incorrect"))
    })?;

    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_valid)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let params = Params::new(65536, 3, 1, Some(32))
        .map_err(|e| AppError::Internal(format!("Failed to create Argon2 params: {e}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt);

    match password_hash {
        Ok(hash) => Ok(hash.to_string()),

        Err(e) => Err(AppError::Internal(format!(
            "Failed to hash password: {}",
            e
        ))),
    }
}

pub fn generate_otp() -> String {
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..999999))
}

pub async fn send_otp_via_whatsapp(
    to: &str,
    from: &str,
    auth_token: &str,
    account_sid: &str,
    otp: &str,
) -> Result<(), AppError> {
    let to_whatsapp = format!("whatsapp:{}", to); // because I'm still in dev, still using my test number

    let sender = format!("whatsapp:{}", from);
    let client = Client::new();

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let res = client
        .post(&url)
        .basic_auth(account_sid, Some(auth_token))
        .form(&[
            ("To", to_whatsapp.as_str()),
            ("From", sender.as_str()),
            (
                "Body",
                &format!(
                    "Your account is under process to finish. Here is your OTP for activate your account: {}, don't share to others. Only valid in 2 minutes.",
                    otp
                ),
            ),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to send message, {}", e)))?;

    if res.status().is_success() {
        info!("OTP successfully send to...");
    }
    Ok(())
}
