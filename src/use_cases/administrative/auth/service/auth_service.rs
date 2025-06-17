use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::Utc;
use entity::{employees, users};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

use crate::{
    dtos::administrative::auth::{
        auth_request::{LoginRequest, RegisterUserRequest},
        auth_response::{LoginResponse, RegisterUserResponse},
    },
    error_handling::app_error::AppError,
    use_cases::administrative::auth::{
        contracts::{
            auth_repo_contract::AuthRepoContract, auth_service_contract::AuthServiceContract,
        },
        repo::auth_repo::AuthRepo,
    },
    utils::{
        helpers::verify_password,
        jwt::{Claims, JwtKeys, generate_jwt},
        limiter::{check_and_update_login_attemps, reset_login_attempts},
    },
};

pub struct AuthService;

#[async_trait]
impl AuthServiceContract for AuthService {
    async fn login(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        jwt_keys: JwtKeys,
        payload: LoginRequest,
    ) -> Result<LoginResponse, AppError> {
        check_and_update_login_attemps(redis, &payload.username).await?;

        let user = <AuthRepo as AuthRepoContract>::find_user_by_username(db, &payload.username)
            .await
            .or(Err(AppError::AuthError(format!(
                "Username or password is incorrect"
            ))))?;

        if !user.is_active {
            return Err(AppError::AuthError(format!("Account disabled")));
        }

        let is_password_valid = verify_password(&payload.password, &user.password)?;

        if !is_password_valid {
            return Err(AppError::AuthError(format!(
                "Username or password is incorrect"
            )));
        }

        reset_login_attempts(redis, &payload.username).await;

        let issued_at = Utc::now().timestamp() as usize;
        let expires_at = issued_at + 6 * 60 * 60;

        let claims = Claims {
            sub: user.id,
            role: user.role.to_string(),
            username: user.username.to_string(),
            iat: issued_at,
            exp: expires_at,
        };

        let access_token = generate_jwt(&claims, &jwt_keys)?;

        let _ = <AuthRepo as AuthRepoContract>::update_last_login(db, &payload.username).await?;

        Ok(LoginResponse {
            access_token,
            token_type: "Bearer".to_string(),
        })
    }

    async fn register_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        claims: Claims,
        payload: RegisterUserRequest,
    ) -> Result<RegisterUserResponse, AppError> {
        // 1. check if employee_id is registered
        let employee = employees::Entity::find()
            .filter(employees::Column::Id.eq(payload.employee_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Employee Id is not found")))?;
        // 2. check if user already registered, if yes, then abort
        let user = users::Entity::find()
            .select_only()
            .column(users::Column::Id)
            .filter(users::Column::EmployeeId.eq(employee.id))
            .one(db)
            .await?;

        if user.is_some() {
            return Err(AppError::Conflict(format!("Employee already sign a user")));
        }
        // 3. check if role related to department
        todo!()
        // 4. create user
        // 5. send otp and return
    }
}
