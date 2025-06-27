use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Response},
};
use tower::{Layer, Service};
use tracing::{Instrument, info_span};

use crate::{error_handling::app_error::AppError, state::AppState, utils::jwt::decode_jwt};

#[derive(Clone)]
pub struct JwtAuthLayer {
    pub app_state: AppState,
}

impl<S> Layer<S> for JwtAuthLayer {
    type Service = JwtAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JwtAuthMiddleware {
            inner,
            app_state: self.app_state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct JwtAuthMiddleware<S> {
    inner: S,
    app_state: AppState,
}

impl<S> Service<Request<Body>> for JwtAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let _ = self.inner.poll_ready(cx);
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let token_result = req
            .headers()
            .get("Authorization")
            .and_then(|val| val.to_str().ok())
            .filter(|s| s.starts_with("Bearer "))
            .map(|s| s.trim_start_matches("Bearer ").trim().to_string());

        let jwt_keys = self.app_state.jwt_keys.clone();
        let mut inner = self.inner.clone();

        let span = info_span!("auth_middleware", uri = %req.uri());

        Box::pin(
            async move {
                let token = match token_result {
                    Some(t) => t,
                    None => {
                        let res =
                            AppError::AuthError("Missing or invalid token".into()).into_response();
                        return Ok(res);
                    }
                };

                let claims = match decode_jwt(&token, &jwt_keys) {
                    Ok(c) => c,
                    Err(_) => {
                        let res =
                            AppError::AuthError("Invalid or expired token".into()).into_response();
                        return Ok(res);
                    }
                };

                req.extensions_mut().insert(claims);

                match inner.call(req).await {
                    Ok(res) => Ok(res),
                    Err(_) => {
                        Ok(AppError::Internal("Internal server error".into()).into_response())
                    }
                }
            }
            .instrument(span),
        )
    }
}
