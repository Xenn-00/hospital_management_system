use axum::{Extension, Json, extract::State};
use validator::Validate;

use crate::{
    dtos::administrative::employment::{
        employment_request::RegisterEmployeeRequest, employment_response::RegisterEmployeeResponse,
    },
    error_handling::app_error::AppError,
    infra::api::ApiResponse,
    middleware::fn_middleware::request_middleware::RequestId,
    state::AppState,
    use_cases::administrative::employee::{
        contracts::employee_service_contract::EmployeeServiceContract,
        service::employee_service::EmployeeService,
    },
    utils::jwt::Claims,
};

pub async fn register_employee(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<RegisterEmployeeRequest>,
) -> Result<Json<ApiResponse<RegisterEmployeeResponse>>, AppError> {
    payload.validate().map_err(AppError::from)?;
    payload.validate_cross_fields().map_err(AppError::from)?;

    let db = &state.db;

    let result =
        <EmployeeService as EmployeeServiceContract>::register_employee(db, payload, claims)
            .await?;

    let response = ApiResponse {
        message: "Register employee success".to_string(),
        data: Some(result),
        request_id: request_id.0.clone(),
        errors: None,
    };

    Ok(Json(response))
}
