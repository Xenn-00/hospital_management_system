use axum::{body::Body, http::Request, middleware::Next, response::IntoResponse};
use http::HeaderValue;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn assign_request_id(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());
    response
}
