use std::convert::Infallible;

use aws_sdk_s3::{error::SdkError, operation::put_object::PutObjectError};
use axum::{
    Json,
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use bb8::RunError;
use http::StatusCode;
use redis::RedisError;
use thiserror::Error;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors};

use crate::infra::{
    api::{ApiFieldError, ApiResponse},
    config::REQUEST_ID,
};

#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error")]
    ValidationErrors(Vec<ApiFieldError>),

    #[error("Validation error")]
    ValidationError(ValidationError),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Too many request error: {0}")]
    TooManyRequests(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

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
        let (status, message, errors) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, None),

            AppError::ValidationErrors(api_field_errors) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".into(),
                Some(api_field_errors),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),

            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg, None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, None),

            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, None),
            AppError::ValidationError(error) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".into(),
                Some(vec![ApiFieldError {
                    field: "".to_string(),
                    message: error
                        .message
                        .clone()
                        .unwrap_or_else(|| error.code.to_string().into())
                        .to_string(),
                }]),
            ),
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

impl From<Infallible> for AppError {
    fn from(value: Infallible) -> Self {
        tracing::error!("server error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        // default fallback
        tracing::error!("server error: {:?}", value);
        AppError::Internal(value.to_string())
    }
}

impl From<validator::ValidationError> for AppError {
    fn from(value: validator::ValidationError) -> Self {
        AppError::ValidationError(value.to_owned())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        AppError::ValidationErrors(AppError::flatten_validation_errors(value))
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

impl From<aws_sdk_s3::Error> for AppError {
    fn from(value: aws_sdk_s3::Error) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<SdkError<PutObjectError>> for AppError {
    fn from(value: SdkError<PutObjectError>) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<MultipartError> for AppError {
    fn from(value: MultipartError) -> Self {
        AppError::BadRequest(value.to_string())
    }
}

impl From<&AppError> for AppError {
    fn from(value: &AppError) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        AppError::Internal(value.to_string())
    }
}
