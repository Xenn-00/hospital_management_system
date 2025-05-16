use axum::{
    Extension, Json,
    extract::{Path, State},
};

use validator::Validate;

use crate::{
    dtos::triage::{
        create_triage_request::CreateTriageRequest,
        response::{
            CreateTriageResponse, TriagePatientCalled, TriagePatientCancel, TriageQueueComplete,
            TriageQueueResponse, TriageQueueStatus,
        },
    },
    error_handling::app_error::AppError,
    infra::api::ApiResponse,
    middleware::request_middleware::RequestId,
    state::AppState,
    use_cases::triage::service::triage_service::{self, TriageService},
};

pub async fn triage_patient(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateTriageRequest>,
) -> Result<Json<ApiResponse<CreateTriageResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;

    let db = &state.db;

    let result =
        <TriageService as triage_service::TriageContracts>::perform_triage(db, payload).await?;

    let response = ApiResponse {
        message: "Triage successful".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };
    Ok(Json(response))
}

pub async fn triage_queue(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(visit_type): Path<String>,
) -> Result<Json<ApiResponse<TriageQueueResponse>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result =
        <TriageService as triage_service::TriageContracts>::get_triage_queue(db, redis, visit_type)
            .await?;

    let response = ApiResponse {
        message: "Get triage queue successful".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };
    Ok(Json(response))
}

pub async fn triage_queue_status(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path((visit_type, queue_number)): Path<(String, i32)>,
) -> Result<Json<ApiResponse<TriageQueueStatus>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result = <TriageService as triage_service::TriageContracts>::get_triage_queue_status_by_id(
        db,
        redis,
        visit_type,
        queue_number,
    )
    .await?;

    let response = ApiResponse {
        message: "Get triage queue status successful".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };
    Ok(Json(response))
}

pub async fn triage_call_patient(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path((visit_type, queue_number)): Path<(String, i32)>,
) -> Result<Json<ApiResponse<TriagePatientCalled>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result = <TriageService as triage_service::TriageContracts>::call_patient(
        db,
        redis,
        visit_type,
        queue_number,
    )
    .await?;

    let response = ApiResponse {
        message: "Call patient successful".to_string(),
        data: Some(result),
        request_id: request_id.0.to_string(),
        errors: None,
    };

    Ok(Json(response))
}

pub async fn triage_complete(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path((visit_type, queue_number)): Path<(String, i32)>,
) -> Result<Json<ApiResponse<TriageQueueComplete>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result = <TriageService as triage_service::TriageContracts>::triage_complete(
        db,
        redis,
        visit_type,
        queue_number,
    )
    .await?;

    let response = ApiResponse {
        message: "Triage patient complete".to_string(),
        data: Some(result),
        request_id: request_id.0.to_string(),
        errors: None,
    };

    Ok(Json(response))
}

pub async fn triage_patient_cancel(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path((visit_type, queue_number)): Path<(String, i32)>,
) -> Result<Json<ApiResponse<TriagePatientCancel>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result = <TriageService as triage_service::TriageContracts>::cancel_patient_queue(
        db,
        redis,
        visit_type,
        queue_number,
    )
    .await?;

    let response = ApiResponse {
        message: "Cancel triage patient complete".to_string(),
        data: Some(result),
        request_id: request_id.0.to_string(),
        errors: None,
    };

    Ok(Json(response))
}
