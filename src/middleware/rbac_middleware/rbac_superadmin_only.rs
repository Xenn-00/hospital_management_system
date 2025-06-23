use axum::{body::Body, extract::Request, middleware::Next, response::Response};

use crate::{error_handling::app_error::AppError, utils::jwt::Claims};

pub async fn rbac_superadmin_only(req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(AppError::AuthError("Missing claims".into()))?;

    if claims.role != "SUPERADMIN" {
        return Err(AppError::Forbidden("Superadmin only".into()));
    }

    Ok(next.run(req).await)
}
