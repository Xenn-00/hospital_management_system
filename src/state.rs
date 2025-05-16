use std::time::Duration;

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

type RedisPool = Pool<RedisConnectionManager>;
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: RedisPool,
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
