use async_trait::async_trait;

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::Utc;
use entity::{
    department_roles, employees, role,
    users::{self, AccountStatus},
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityOrSelect, EntityTrait,
    QueryFilter, QuerySelect, TransactionTrait,
};
use uuid::Uuid;

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
    use_cases::administrative::auth::{
        contracts::{
            auth_repo_contract::AuthRepoContract, auth_service_contract::AuthServiceContract,
        },
        repo::auth_repo::AuthRepo,
    },
    utils::{
        helpers::{
            delete_cache_data, generate_otp, get_cache_data, hash_password, send_otp_via_whatsapp,
            set_cache_data, verify_password,
        },
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
        jwt_keys: &JwtKeys,
        payload: LoginRequest,
    ) -> Result<LoginResponse, AppError> {
        check_and_update_login_attemps(redis, &payload.username).await?;

        let user = <AuthRepo as AuthRepoContract>::find_user_by_username(db, &payload.username)
            .await
            .or(Err(AppError::AuthError(format!(
                "Username or password is incorrect"
            ))))?;

        if user.account_status != AccountStatus::Active {
            return Err(AppError::AuthError(format!("Account disabled")));
        }

        let user_password = user
            .password
            .as_ref()
            .ok_or(AppError::AuthError("User password not set".to_string()))?
            .clone();

        let verified =
            tokio::task::spawn_blocking(move || verify_password(&payload.password, &user_password))
                .await
                .map_err(|_| AppError::Internal(format!("Password verification failed")))?;

        let is_password_valid = verified?;

        if !is_password_valid {
            return Err(AppError::AuthError(format!(
                "Username or password is incorrect"
            )));
        }

        reset_login_attempts(redis, &payload.username).await;

        let user_role = role::Entity::find_by_id(user.role_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("Unknown user role")))?;

        let issued_at = Utc::now().timestamp() as usize;
        let expires_at = issued_at + 6 * 60 * 60;

        let claims = Claims {
            sub: user.id,
            role: user_role.name,
            username: user.username.unwrap().to_string(),
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
        twilio: &Twilio,
        claims: Claims,
        payload: AdminCreateEmployeeAccountRequest,
    ) -> Result<AdminCreateEmployeeAccountResponse, AppError> {
        // 1. check if employee_id is registered
        let employee_id = payload.employee_id;
        let employee = employees::Entity::find()
            .filter(employees::Column::Id.eq(employee_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Employee Id is not found")))?;

        // 2. check if user already registered, if yes, then abort
        let user_exists = users::Entity::find()
            .filter(users::Column::EmployeeId.eq(employee.id))
            .select()
            .column(users::Column::Id)
            .one(db)
            .await?
            .is_some();

        if user_exists {
            return Err(AppError::Conflict(
                "Employee already sign a user".to_string(),
            ));
        }
        // 3. check if role related to department
        let role_request = payload.role;

        let check_role = role::Entity::find()
            .filter(role::Column::Code.eq(&role_request))
            .join(
                sea_orm::JoinType::InnerJoin,
                role::Entity::belongs_to(department_roles::Entity)
                    .from(role::Column::Id)
                    .to(department_roles::Column::RoleId)
                    .into(),
            )
            .filter(department_roles::Column::DepartmentCode.eq(employee.department_code))
            .one(db)
            .await;

        let role = match check_role {
            Ok(Some(role)) => role,
            Ok(None) => {
                return Err(AppError::BadRequest(format!(
                    "Role is invalid. Request info: {:?}",
                    &role_request
                )));
            }
            Err(e) => {
                tracing::warn!("Error: {}", e);
                return Err(AppError::Internal(format!("Database error: {}", e)));
            }
        };

        // 4. create user
        let creator_id = claims.sub;
        let new_user = users::ActiveModel {
            employee_id: Set(employee.id),
            created_by: Set(Some(creator_id)),
            role_id: Set(role.id),
            ..Default::default()
        };

        let txn = db.begin().await?;
        let register = <AuthRepo as AuthRepoContract>::register_new_user(&txn, new_user).await?;
        txn.commit().await?;

        // 5. send otp and return
        let otp = generate_otp();

        let to = &twilio.to; // still in dev and have no enough money to buy twilio pro 🫠, in prod we should change it to actual employee phone number
        let from = &twilio.whatsapp_sandbox; // still in dev and have no enough money to buy twilio pro 🫠
        let account_sid = &twilio.account_sid;
        let auth_token = &twilio.auth_token;

        let cache_key = format!("otp:{}", &register.employee_id);
        set_cache_data(&redis, &cache_key, &otp, 120).await?;
        send_otp_via_whatsapp(to, from, auth_token, account_sid, &otp).await?;

        Ok(AdminCreateEmployeeAccountResponse {
            employee_id: register.employee_id,
            role: role.code,
            status: format!("{:?}", register.account_status),
        })
    }
    async fn verify_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        payload: VerifyOtpRequest,
    ) -> Result<VerifyOtpResponse, AppError> {
        let employee_id = payload.employee_id;

        let check_user = users::Entity::find()
            .filter(users::Column::EmployeeId.eq(employee_id))
            .one(db)
            .await?;

        let user = check_user.ok_or_else(|| AppError::NotFound("User not found".into()))?;

        if user.account_status != AccountStatus::PendingVerification {
            return Err(AppError::BadRequest(format!("Account already verified")));
        }

        // 1. check redis, seek for employee otp
        let cache_key = format!("otp:{}", employee_id);

        let Some(cached_otp) = get_cache_data::<String>(redis, &cache_key).await? else {
            tracing::warn!("Cannot get OTP.");
            return Err(AppError::BadRequest(
                "OTP is invalid or already expired".to_string(),
            ));
        };
        // 2. verify is the otp valid
        if cached_otp != payload.otp_number {
            tracing::warn!("OTP mismatch for employee_id {}", employee_id);
            return Err(AppError::BadRequest(
                "OTP is invalid or already expired".to_string(),
            ));
        }

        delete_cache_data(redis, &cache_key).await?;

        // 3. update account status
        let mut active_model: users::ActiveModel = user.into();
        active_model.account_status = Set(users::AccountStatus::AwaitingSetup);

        users::Entity::update(active_model).exec(db).await?;
        // 4. set temporary token for employee to proceed
        let setup_token = Uuid::new_v4().to_string();
        let setup_cache_key = format!("setup:{}", employee_id);
        set_cache_data(redis, &setup_cache_key, &setup_token, 300).await?;

        Ok(VerifyOtpResponse {
            message: format!(
                "Account verified. Please set your username and password, then you're all set"
            ),
            employee_id,
            temporary_setup_token: setup_token,
        })
    }

    async fn setup_user(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        payload: EmployeeRegisterUserRequest,
        setup_token: String,
        employee_id: i32,
    ) -> Result<EmployeeRegisterUserResponse, AppError> {
        // 1. check setup token and user model
        let check_user = users::Entity::find()
            .filter(users::Column::EmployeeId.eq(employee_id))
            .one(db)
            .await?;

        let user = check_user.ok_or_else(|| AppError::NotFound("User not found".into()))?;

        let cache_key = format!("setup:{}", employee_id);
        let Some(cached_setup_token) = get_cache_data::<String>(redis, &cache_key).await? else {
            tracing::warn!("Cannot get setup token.");
            return Err(AppError::BadRequest(format!(
                "Setup key is invalid or already expired"
            )));
        };

        if cached_setup_token != setup_token {
            tracing::warn!("Setup token mismatch for employee_id {}", employee_id);
            return Err(AppError::BadRequest(format!(
                "Setup token is invalid or already expired"
            )));
        }
        // 2. proceed the request
        let password_hash = tokio::task::spawn_blocking(move || hash_password(&payload.password))
            .await
            .map_err(|_| AppError::Internal(format!("Failed to hash password")))?;

        let hashed = password_hash?;

        // 3. set all
        let mut active_model: users::ActiveModel = user.into();
        active_model.username = Set(Some(payload.username));
        active_model.password = Set(Some(hashed));
        active_model.account_status = Set(AccountStatus::Active);

        users::Entity::update(active_model)
            .exec(db)
            .await
            .map_err(|e| {
                if let DbErr::Exec(ref err) = e {
                    if err.to_string().contains("unique_username") {
                        return AppError::Conflict("Username is already taken".into());
                    }
                }
                AppError::Internal(format!("Unexpected DB error: {}", e))
            })?;

        delete_cache_data(redis, &cache_key).await?;
        // 4. return

        Ok(EmployeeRegisterUserResponse {
            message: format!("Your account is all set"),
            status: format!("{:?}", AccountStatus::Active),
            redirect_url: format!("https://app.hospital.internal/auth/login"), // hopefully in the future can afford domain name 🫠
        })
    }
}
