use crate::{error_handling::app_error::AppError, state::AppState, utils::jwt::decode_jwt};
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

pub async fn jwt_auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(AppError::AuthError("Missing token".into()))?
        .to_str()
        .map_err(|_| AppError::AuthError("Invalid token".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(AppError::AuthError("Invalid token format".into()))?;

    let claims = decode_jwt(token, &app_state.jwt_keys)
        .map_err(|_| AppError::AuthError("Invalid token".into()))?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
