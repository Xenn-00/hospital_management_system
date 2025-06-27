use std::sync::Arc;

use axum::{Router, middleware, routing::post};

use crate::{
    handlers::administrative::employment::employee_handler::register_employee,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        rbac_middleware::rbac_superadmin_only::rbac_superadmin_only,
    },
    state::AppState,
};

pub fn employment_routes(app_state: Arc<AppState>) -> Router<AppState> {
    Router::new()
        .route("/register", post(register_employee))
        .layer(middleware::from_fn_with_state(
            app_state,
            rbac_superadmin_only,
        ))
        .layer(middleware::from_fn(assign_request_id))
}
