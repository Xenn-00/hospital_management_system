use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::administrative::auth::{
        auth_request::{
            AdminCreateEmployeeAccountRequest, EmployeeRegisterUserRequest, LoginRequest,
            VerifyOtpRequest,
        },
        auth_response::{
            AdminCreateEmployeeAccountResponse, EmployeeRegisterUserResponse, LoginResponse,
            VerifyOtpResponse,
        },
    },
    error_handling::app_error::AppError,
    infra::config::Twilio,
    utils::jwt::{Claims, JwtKeys},
};

#[async_trait]
pub trait AuthServiceContract {
    async fn login(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        jwt_keys: &JwtKeys,
        payload: LoginRequest,
    ) -> Result<LoginResponse, AppError>;
    async fn register_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        twilio: &Twilio,
        claims: Claims,
        payload: AdminCreateEmployeeAccountRequest,
    ) -> Result<AdminCreateEmployeeAccountResponse, AppError>;
    async fn verify_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        payload: VerifyOtpRequest,
    ) -> Result<VerifyOtpResponse, AppError>;
    async fn setup_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        payload: EmployeeRegisterUserRequest,
        setup_token: String,
        employee_id: i32,
    ) -> Result<EmployeeRegisterUserResponse, AppError>;
}
