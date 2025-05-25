use axum::extract::multipart::Field;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::{StreamExt, TryStreamExt};
use image::ImageReader;
use serde::de::DeserializeOwned;

use crate::error_handling::app_error::AppError;

use redis::AsyncCommands;

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
