use axum::{Extension, Json, extract::State};
use validator::Validate;

use crate::{
    dtos::administrative::auth::{
        auth_request::{LoginRequest, RegisterUserRequest},
        auth_response::{LoginResponse, RegisterUserResponse},
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
        <AuthService as AuthServiceContract>::login(db, redis, state.jwt_keys, payload).await?;

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
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<ApiResponse<RegisterUserResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;
    let redis = &state.redis;

    let result =
        <AuthService as AuthServiceContract>::register_user(db, redis, claims, payload).await?;

    let response = ApiResponse {
        message: "Register user success".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}
