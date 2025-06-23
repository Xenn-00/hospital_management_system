use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use tracing::{error, warn};

use crate::error_handling::app_error::AppError;

const MAX_ATTEMPTS: u32 = 7;
const BLOCK_DURATION_SECS: i64 = 3600; // 60 minutes

fn make_login_attempt_key(username: &str) -> String {
    format!("auth:login_attempts:{}", username)
}

pub async fn check_and_update_login_attemps(
    pool: &Pool<RedisConnectionManager>,
    username: &str,
) -> Result<(), AppError> {
    let mut conn = pool.get().await.map_err(|e| {
        error!("Failed to get redis connection: {:?}", e);
        AppError::Internal("Internal Server Error".into())
    })?;

    let key = make_login_attempt_key(username);
    let attempts_result = conn.get(&key).await;
    let current_attempts = match attempts_result {
        Ok(Some(count)) => count,
        Ok(None) => 0,
        Err(e) => {
            error!("Failed to fetch into redis for key {}: {:?}", key, e);
            return Err(AppError::Internal("Intenal server error".into()));
        }
    };

    if current_attempts >= MAX_ATTEMPTS {
        return Err(AppError::TooManyRequests(format!(
            "Too many failed login attempts. Try again later."
        )));
    }

    let _: () = redis::pipe()
        .atomic()
        .incr(&key, 1)
        .expire(&key, BLOCK_DURATION_SECS)
        .query_async(&mut *conn)
        .await
        .map_err(|e| {
            warn!("Failed to increment Redis login attempts: {:?}", e);
            AppError::Internal("Internal server error".into())
        })?;

    Ok(())
}

pub async fn reset_login_attempts(pool: &Pool<RedisConnectionManager>, username: &str) {
    let key = make_login_attempt_key(username);

    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get redis connection: {:?}", e);
            return;
        }
    };

    if let Err(e) = conn.del::<_, ()>(&key).await {
        warn!("Failed to delete key login attempts '{}': {:?}", key, e);
    }
}
