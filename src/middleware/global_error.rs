use crate::error_handling::app_error::AppError;
use crate::middleware::request_middleware::RequestId;
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

// async fn handle_global_error(
//     err: Box<dyn std::error::Error + Send + Sync>,
//     req: Request<Body>,
// ) -> impl IntoResponse {
//     let request_id = req
//         .extensions()
//         .get::<RequestId>()
//         .map(|r| r.0.clone())
//         .unwrap_or_else(|| "unknown".to_string());

//     let app_error = AppError::Internal(err.to_string());
//     app_error.into_response(request_id)
// }

pub async fn error_handling_layer(req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    match next.run(req).await.into_response().into_parts() {
        (parts, _body) if parts.status.is_server_error() => {
            let err = AppError::Internal("Internal server error".to_string());
            Ok(err.into_response(request_id).into_response())
        }
        (parts, body) => Ok(Response::from_parts(parts, body)),
    }
}
