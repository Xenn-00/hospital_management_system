use std::time::Duration;

use aws_config::Region;
use aws_sdk_s3::{
    Client,
    config::{Builder, Credentials, SharedCredentialsProvider},
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::infra::config::S3Config;

type RedisPool = Pool<RedisConnectionManager>;
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: RedisPool,
    pub s3: Client,
}

pub async fn init_database_connection(url: &str) -> DatabaseConnection {
    let mut options = ConnectOptions::new(url);
    options
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    Database::connect(options)
        .await
        .expect("Failed to connect to the database!")
}

pub async fn init_redis_pool(redis_url: &str) -> RedisPool {
    let manager =
        RedisConnectionManager::new(redis_url).expect("Failed to connect to redis server");

    Pool::builder()
        .max_size(20)
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
