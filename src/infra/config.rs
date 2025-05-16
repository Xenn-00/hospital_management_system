use config::{Config, ConfigError, File};
use serde::{Deserialize, de::DeserializeOwned};
use tokio::task_local;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub app: Application,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub driver: String,
    pub direct_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub official_url: String,
    pub upstash_redis_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Application {
    pub host: String,
    pub port: i16,
}

impl AppConfig {
    pub fn from_yaml(path: &str) -> Result<Self, config::ConfigError> {
        let builder = Config::builder().add_source(File::with_name(path));

        builder.build()?.try_deserialize()
    }

    pub fn get_key<T: DeserializeOwned>(path: &str, file: &str) -> Result<T, ConfigError> {
        Config::builder()
            .add_source(File::with_name(file))
            .build()?
            .get::<T>(path)
    }
}

task_local! {
    pub static REQUEST_ID: String
}
