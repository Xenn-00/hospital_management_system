use axum::{
    Json,
    response::{IntoResponse, Response},
};
use bb8::RunError;
use http::StatusCode;
use redis::RedisError;
use thiserror::Error;
use uuid::Uuid;
use validator::ValidationErrors;

use crate::infra::{
    api::{ApiFieldError, ApiResponse},
    config::REQUEST_ID,
};

#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error")]
    ValidationError(Vec<ApiFieldError>),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn flatten_validation_errors(errors: ValidationErrors) -> Vec<ApiFieldError> {
        errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let message = errors
                    .iter()
                    .map(|e| {
                        e.message
                            .clone()
                            .unwrap_or_else(|| e.code.to_string().into())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                ApiFieldError {
                    field: field.to_string(),
                    message,
                }
            })
            .collect()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self.clone() {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            AppError::ValidationError(api_field_errors) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".into(),
                Some(api_field_errors.clone()),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), None),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), None),
        };

        let request_id = REQUEST_ID
            .try_with(|id| id.clone())
            .unwrap_or_else(|_| Uuid::new_v4().to_string());

        let response = ApiResponse::<()> {
            message,
            data: None,
            errors,
            request_id,
        };

        (status, Json(response)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        // default fallback
        tracing::error!("server error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        AppError::ValidationError(AppError::flatten_validation_errors(value))
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(value: sea_orm::DbErr) -> Self {
        tracing::error!("DB error: {:?}", value);
        AppError::Internal(format!("DB error: {}", value))
    }
}

impl From<RunError<RedisError>> for AppError {
    fn from(value: RunError<RedisError>) -> Self {
        tracing::error!("Redis error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}

impl From<RedisError> for AppError {
    fn from(value: RedisError) -> Self {
        tracing::error!("Redis error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        tracing::error!("serde error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}
