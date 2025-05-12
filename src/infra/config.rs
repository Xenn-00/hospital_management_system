use config::{Config, ConfigError, File};
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub driver: String,
    pub direct_url: String,
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
