use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::administrative::auth::{
        auth_request::{LoginRequest, RegisterUserRequest},
        auth_response::{LoginResponse, RegisterUserResponse},
    },
    error_handling::app_error::AppError,
    utils::jwt::{Claims, JwtKeys},
};

#[async_trait]
pub trait AuthServiceContract {
    async fn login(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        jwt_keys: JwtKeys,
        payload: LoginRequest,
    ) -> Result<LoginResponse, AppError>;
    async fn register_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        claims: Claims,
        payload: RegisterUserRequest,
    ) -> Result<RegisterUserResponse, AppError>;
}
