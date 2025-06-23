use axum::{
    Extension, Json,
    extract::{Query, State},
};
use validator::Validate;

use crate::{
    dtos::administrative::auth::{
        auth_request::{
            AdminCreateEmployeeAccountRequest, EmployeeRegisterUserRequest, LoginRequest,
            SetupUserQuery, VerifyOtpRequest,
        },
        auth_response::{
            AdminCreateEmployeeAccountResponse, EmployeeRegisterUserResponse, LoginResponse,
            VerifyOtpResponse,
        },
    },
    error_handling::app_error::AppError,
    infra::api::ApiResponse,
    middleware::fn_middleware::request_middleware::RequestId,
    state::AppState,
    use_cases::administrative::auth::{
        contracts::auth_service_contract::AuthServiceContract, service::auth_service::AuthService,
    },
    utils::jwt::Claims,
};

pub async fn login_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;
    let redis = &state.redis;

    let result =
        <AuthService as AuthServiceContract>::login(db, redis, &state.jwt_keys, payload).await?;

    let response = ApiResponse {
        message: "Login successful".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}

pub async fn register_handler(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AdminCreateEmployeeAccountRequest>,
) -> Result<Json<ApiResponse<AdminCreateEmployeeAccountResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;
    let redis = &state.redis;
    let twilio = &state.twilio;

    let result =
        <AuthService as AuthServiceContract>::register_user(db, redis, twilio, claims, payload)
            .await?;

    let response = ApiResponse {
        message: "Register user success".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}

pub async fn verify_otp(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<ApiResponse<VerifyOtpResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;
    let redis = &state.redis;

    let result = <AuthService as AuthServiceContract>::verify_user(db, redis, payload).await?;

    let response = ApiResponse {
        message: "User verified".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };
    Ok(Json(response))
}

pub async fn employee_user_setup(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Query(setup): Query<SetupUserQuery>,
    Json(payload): Json<EmployeeRegisterUserRequest>,
) -> Result<Json<ApiResponse<EmployeeRegisterUserResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;
    let redis = &state.redis;

    let SetupUserQuery {
        setup_token,
        employee_id,
    } = setup;

    let result = <AuthService as AuthServiceContract>::setup_user(
        db,
        redis,
        payload,
        setup_token,
        employee_id,
    )
    .await?;

    let response = ApiResponse {
        message: "Setup complete".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}
