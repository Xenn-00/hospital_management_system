use std::sync::Arc;

use axum::{Router, middleware, routing::post};

use crate::{
    handlers::administrative::auth::auth_handler::login_handler,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::error_handler_layer::ErrorHandlingLayer,
    },
    state::AppState,
};

pub fn auth_routes(app_state: Arc<AppState>) -> Router<AppState> {
    Router::with_state(Router::new(), app_state)
        .layer(middleware::from_fn(assign_request_id))
        .route("/auth/login", post(login_handler))
        .layer(ErrorHandlingLayer)
}
