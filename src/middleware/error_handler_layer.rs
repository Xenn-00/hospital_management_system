use std::pin::Pin;

use axum::{
    BoxError,
    extract::Request,
    response::{IntoResponse, Response},
};
use tower::{Layer, Service};
use tracing::{Instrument, info_span};
use uuid::Uuid;

use log::error;

use crate::error_handling::app_error::AppError;

use super::request_middleware::RequestId;

#[derive(Clone)]
pub struct ErrorHandlingLayer;

impl<S> Layer<S> for ErrorHandlingLayer {
    type Service = ErrorHandlingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorHandlingMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ErrorHandlingMiddleware<S> {
    inner: S,
}

impl<ReqBody, S> Service<Request<ReqBody>> for ErrorHandlingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| RequestId(Uuid::new_v4().to_string()));
        let mut inner = self.inner.clone();
        let span = info_span!("request", method = %req.method(), uri = %req.uri(), request_id = ?request_id);

        Box::pin(
            async move {
                let response = inner.call(req).await;

                match response {
                    Ok(res) => Ok(res),
                    Err(err) => {
                        let app_err: AppError = AppError::Internal(format!("{:?}", err.into()));
                        error!("Error caught in middleware: {}", app_err);
                        Ok(app_err.into_response())
                    }
                }
            }
            .instrument(span),
        )
    }
}
