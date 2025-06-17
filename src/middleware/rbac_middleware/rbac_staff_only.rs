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

    if claims.role != "staff"
        && claims.role != "triage_staff"
        && claims.role != "nurse"
        && claims.role != "Superadmin"
    {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    Ok(next.run(req).await)
}
