use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{patch, post},
};

use crate::{
    handlers::administrative::auth::auth_handler::{
        employee_user_setup, login_handler, register_handler, resend_otp, verify_otp,
    },
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        rbac_middleware::rbac_superadmin_only::rbac_superadmin_only,
    },
    state::AppState,
};

pub fn auth_routes_protected(app_state: Arc<AppState>) -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .layer(middleware::from_fn_with_state(
            app_state,
            rbac_superadmin_only,
        ))
        .layer(middleware::from_fn(assign_request_id))
}

pub fn auth_routes_public() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/resend-otp", post(resend_otp))
        .route("/verify-otp", post(verify_otp))
        .route("/register/setup", patch(employee_user_setup))
        .layer(middleware::from_fn(assign_request_id))
}
