use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{error_handling::app_error::AppError, state::AppState, utils::jwt::Claims};

#[allow(unused_mut)]
pub async fn rbac_staff_only(
    State(_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(AppError::AuthError("Missing claims".into()))?;

    if claims.role == "SUPERADMIN"
        || claims.role == "ADMIN_GENERAL"
        || claims.role == "FRONT_STAFF"
        || claims.role == "DEPARTMENT_HEAD"
        || claims.role == "EMERGENCY_STAFF"
        || claims.role == "SUPPORT_STAFF"
    {
        return Ok(next.run(req).await);
    }

    Err(AppError::Forbidden("Insufficient permissions".into()))
}
