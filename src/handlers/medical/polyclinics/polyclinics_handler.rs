use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};

use crate::{
    dtos::medical::medical_response::{
        PagedPolyclinicSchedulesResponse, PolyclinicSchedulesResponse,
    },
    error_handling::app_error::AppError,
    infra::api::{ApiResponse, PaginationQuery},
    middleware::fn_middleware::request_middleware::RequestId,
    state::AppState,
    use_cases::medical::polyclinic::{
        contracts::polyclinic_service_contract::PolyclinicServiceContract,
        service::polyclinic_service::PolyclinicService,
    },
};

pub async fn get_polyclinic_schedules_by_code(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(poly_code): Path<String>,
) -> Result<Json<ApiResponse<PolyclinicSchedulesResponse>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result = <PolyclinicService as PolyclinicServiceContract>::polyclinic_schedules(
        db, redis, &poly_code,
    )
    .await?;

    let response = ApiResponse {
        message: format!("Schedules for {} retrieved successfully", poly_code),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}

pub async fn get_polyclinic_schedules(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Query(paging): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<PagedPolyclinicSchedulesResponse>>, AppError> {
    let db = &state.db;
    let redis = &state.redis;

    let result =
        <PolyclinicService as PolyclinicServiceContract>::get_all_schedules(db, redis, paging)
            .await?;

    let response = ApiResponse {
        message: "All polyclinic schedules retrieved successfully".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}
