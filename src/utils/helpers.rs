use bb8::Pool;
use bb8_redis::RedisConnectionManager;
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
