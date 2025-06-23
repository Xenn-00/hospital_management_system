use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{patch, post},
};

use crate::{
    handlers::administrative::auth::auth_handler::{
        employee_user_setup, login_handler, register_handler, verify_otp,
    },
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        rbac_middleware::rbac_superadmin_only::rbac_superadmin_only,
    },
    state::AppState,
};

pub fn auth_routes_protected(app_state: Arc<AppState>) -> Router<AppState> {
    Router::with_state(Router::new(), app_state.clone())
        .route("/register", post(register_handler))
        .layer(middleware::from_fn_with_state(
            app_state,
            rbac_superadmin_only,
        ))
        .layer(middleware::from_fn(assign_request_id))
}

pub fn auth_routes_public(app_state: Arc<AppState>) -> Router<AppState> {
    Router::with_state(Router::new(), app_state.clone())
        .route("/login", post(login_handler))
        .route("/verify-otp", post(verify_otp))
        .route("/register/setup", patch(employee_user_setup))
        .layer(middleware::from_fn(assign_request_id))
}
