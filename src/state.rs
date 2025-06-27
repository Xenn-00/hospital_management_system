use std::{sync::Arc, time::Duration};

use aws_config::Region;
use aws_sdk_s3::{
    Client,
    config::{Builder, Credentials, SharedCredentialsProvider},
};
use axum::extract::FromRef;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::{
    error_handling::app_error::AppError,
    infra::config::{S3Config, Twilio},
    utils::jwt::JwtKeys,
};

type RedisPool = Pool<RedisConnectionManager>;
#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis: Arc<RedisPool>,
    pub s3: Arc<Client>,
    pub jwt_keys: Arc<JwtKeys>,
    pub twilio: Arc<Twilio>,
}

pub async fn init_database_connection(url: &str) -> DatabaseConnection {
    let mut options = ConnectOptions::new(url);
    options
        .max_connections(50)
        .min_connections(10)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300));

    Database::connect(options)
        .await
        .expect("Failed to connect to the database!")
}

pub async fn init_redis_pool(redis_url: &str) -> RedisPool {
    let manager =
        RedisConnectionManager::new(redis_url).expect("Failed to connect to redis server");

    Pool::builder()
        .max_size(50)
        .min_idle(Some(10))
        .idle_timeout(Some(Duration::from_secs(300)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(manager)
        .await
        .expect("Failed to build Redis pool")
}

pub async fn init_s3_client(cfg: &S3Config) -> Client {
    let creds = Credentials::new(&cfg.s3_access_key, &cfg.s3_secret_key, None, None, "static");

    let shared_creds = SharedCredentialsProvider::new(creds);

    let conf = Builder::new()
        .region(Region::new(cfg.s3_region.clone()))
        .credentials_provider(shared_creds)
        .endpoint_url(&cfg.s3_url)
        .force_path_style(true)
        .build();

    Client::from_conf(conf)
}

pub async fn load_jwt_keys() -> Result<JwtKeys, AppError> {
    let private_key = tokio::fs::read("private_key.pem").await?;
    let public_key = tokio::fs::read("public_key.pem").await?;

    Ok(JwtKeys {
        encoding: Arc::new(EncodingKey::from_rsa_pem(&private_key)?),
        decoding: Arc::new(DecodingKey::from_rsa_pem(&public_key)?),
    })
}
