use axum::{
    body::Body, extract::FromRequestParts, http::Request, middleware::Next, response::IntoResponse,
};
use http::HeaderValue;
use std::convert::Infallible;
use uuid::Uuid;

use crate::infra::config::REQUEST_ID;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(existing) = parts.extensions.get::<RequestId>() {
            Ok(existing.clone())
        } else {
            let id = Uuid::new_v4().to_string();
            parts.extensions.insert(RequestId(id.clone()));
            Ok(RequestId(id))
        }
    }

    type Rejection = Infallible;
}

pub async fn assign_request_id(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    REQUEST_ID
        .scope(request_id.clone(), async {
            req.extensions_mut().insert(RequestId(request_id.clone()));
        })
        .await;

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());
    response
}
