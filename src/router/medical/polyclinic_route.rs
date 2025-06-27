use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::{
    handlers::medical::polyclinics::polyclinics_handler::{
        get_polyclinic_schedules, get_polyclinic_schedules_by_code,
    },
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        rbac_middleware::rbac_staff_only::rbac_staff_only,
    },
    state::AppState,
};

pub fn polyclinic_routes(app_state: Arc<AppState>) -> Router<AppState> {
    Router::new()
        .route(
            "/polyclinics/{poly_code}/schedules",
            get(get_polyclinic_schedules_by_code),
        )
        .route("/polyclinics/schedules", get(get_polyclinic_schedules))
        .layer(middleware::from_fn_with_state(app_state, rbac_staff_only))
        .layer(middleware::from_fn(assign_request_id))
}
