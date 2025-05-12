use axum::{Json, response::IntoResponse};
use http::StatusCode;
use thiserror::Error;

use crate::infra::api::{ApiFieldError, ApiResponse};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error")]
    ValdidationError(Vec<ApiFieldError>),

    #[error("Not found")]
    NotFound(String),

    #[error("Internal server error")]
    Internal(String),
}

impl AppError {
    pub fn into_response(self, request_id: String) -> impl IntoResponse {
        match self {
            AppError::ValdidationError(errors) => {
                let response = ApiResponse::<()> {
                    request_id: request_id.to_string(),
                    message: "Validation failed".to_string(),
                    data: None,
                    errors: Some(errors),
                };
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            AppError::NotFound(message) => {
                let response = ApiResponse::<()> {
                    request_id: request_id.to_string(),
                    message: message.clone(),
                    data: None,
                    errors: None,
                };
                (StatusCode::NOT_FOUND, Json(response)).into_response()
            }
            AppError::Internal(message) => {
                let response = ApiResponse::<()> {
                    request_id: request_id.to_string(),
                    message: message.clone(),
                    data: None,
                    errors: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
    }
}
