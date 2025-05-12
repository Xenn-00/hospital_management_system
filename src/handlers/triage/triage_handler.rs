use axum::{Json, extract::State};
use http::StatusCode;
use sea_orm::TransactionTrait;
use validator::Validate;

use crate::{
    dtos::triage::{create_triage_request::CreateTriageRequest, response::CreateTriageResponse},
    error_handling::app_error::AppError,
    state::AppState,
    use_cases::triage::service::triage_service::{self, TriageService},
};

pub async fn triage_patient(
    State(state): State<AppState>,
    Json(payload): Json<CreateTriageRequest>,
) -> Result<Json<CreateTriageResponse>, (StatusCode, String)> {
    payload
        .validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation failed: {}", e)))?;

    let db = &state.db;
    let txn = db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start transaction: {}", e)))
        .unwrap();

    let response =
        <TriageService as triage_service::TriageContracts>::perform_triage(db, payload).await;

    match response {
        Ok(res) => Ok(Json(res)),
        Err(e) => {
            txn.rollback()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to rollback transaction: {}", e)))
                .unwrap();
            Err(AppError::Internal(format!("Error: {}", e))).unwrap()
        }
    }
}
