use axum::middleware;
use axum::routing::patch;
use axum::{Router, routing::get, routing::post};

use crate::handlers::triage::triage_handler::{
    triage_call_patient, triage_complete, triage_patient, triage_patient_cancel, triage_queue,
    triage_queue_status,
};

use crate::middleware::request_middleware::assign_request_id;
use crate::state::AppState;

pub fn triage_routes(app_state: AppState) -> Router {
    Router::new()
        .layer(middleware::from_fn(assign_request_id))
        .route("/triage", post(triage_patient))
        .route("/triage/queue/{visit_type}", get(triage_queue))
        .route(
            "/triage/queue/{visit_type}/{queue_number}",
            get(triage_queue_status),
        )
        .route(
            "/triage/call/{visit_type}/{queue_number}",
            patch(triage_call_patient),
        )
        .route(
            "/triage/complete/{visit_type}/{queue_number}",
            patch(triage_complete),
        )
        .route(
            "/triage/queue/{visit_type}/{queue_number}/cancel",
            patch(triage_patient_cancel),
        )
        .with_state(app_state)
}
