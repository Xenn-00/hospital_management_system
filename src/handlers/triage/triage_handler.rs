use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
};

use log::info;
use validator::Validate;

use crate::{
    dtos::triage::{
        create_triage_request::CreateTriageRequest,
        referral_upload_metadata::ReferralUploadMetadata,
        response::{
            CreateTriageResponse, ReferralUploadResponse, TriagePatientCalled, TriagePatientCancel,
            TriageQueueComplete, TriageQueueResponse, TriageQueueStatus,
        },
    },
    error_handling::app_error::AppError,
    infra::api::ApiResponse,
    middleware::request_middleware::RequestId,
    state::AppState,
    use_cases::triage::service::triage_service::{self, TriageService},
    utils::helpers::read_bytes_from_multipart_field,
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

const MAX_FILE_SIZE: usize = 1024 * 1024 * 10; // 10 MB

pub async fn triage_referral_document_upload(
    State(state): State<AppState>,
    Path((patient_id, visit_id)): Path<(i32, i32)>,
    Extension(request_id): Extension<RequestId>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<ReferralUploadResponse>>, AppError> {
    let mut metadata: Option<ReferralUploadMetadata> = None;
    let db = &state.db;
    let s3 = &state.s3;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("");
        if name != "file" {
            continue;
        }
        let original_filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or("unknown bin".to_string());

        let ext = original_filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        let valid_ext = ["pdf", "png", "jpg", "jpeg"];

        let is_valid = valid_ext.contains(&ext.as_str());
        if !is_valid {
            return Err(AppError::BadRequest("Invalid file extension".to_string()));
        }

        let bytes = read_bytes_from_multipart_field(field, MAX_FILE_SIZE).await?;

        metadata = Some(ReferralUploadMetadata {
            extension: ext,
            original_filename,
            file_bytes: bytes,
        });

        break;
    }

    let meta = match metadata {
        Some(m) => m,
        None => {
            return Err(AppError::BadRequest(
                "No valid file field found in form".to_string(),
            ));
        }
    };

    info!(
        "Uploading file for patient_id: {}, visit_id: {}, filename: {}",
        patient_id, visit_id, &meta.original_filename,
    );

    let result = <TriageService as triage_service::TriageContracts>::handle_referral_upload(
        db, s3, visit_id, patient_id, meta,
    )
    .await?;

    let response = ApiResponse {
        message: "file upload complete".to_string(),
        data: Some(result),
        request_id: request_id.0.to_string(),
        errors: None,
    };

    Ok(Json(response))
}
