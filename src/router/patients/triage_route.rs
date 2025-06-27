use std::sync::Arc;

use axum::middleware;
use axum::routing::patch;
use axum::{Router, routing::get, routing::post};

use crate::handlers::triage::triage_handler::{
    triage_call_patient, triage_complete, triage_patient, triage_patient_cancel, triage_queue,
    triage_queue_status, triage_referral_document_upload,
};
use crate::middleware::fn_middleware::request_middleware::assign_request_id;
use crate::middleware::rbac_middleware::rbac_staff_only::rbac_staff_only;
use crate::state::AppState;

pub fn triage_routes(app_state: Arc<AppState>) -> Router<AppState> {
    Router::new()
        .route("/", post(triage_patient))
        .route("/queue/{visit_type}", get(triage_queue))
        .route(
            "/queue/{visit_type}/{queue_number}",
            get(triage_queue_status),
        )
        .route(
            "/call/{visit_type}/{queue_number}",
            patch(triage_call_patient),
        )
        .route(
            "/complete/{visit_type}/{queue_number}",
            patch(triage_complete),
        )
        .route(
            "/queue/{visit_type}/{queue_number}/cancel",
            patch(triage_patient_cancel),
        )
        .route(
            "/patient/{patient_id}/{visit_id}/upload",
            post(triage_referral_document_upload),
        )
        .layer(middleware::from_fn_with_state(app_state, rbac_staff_only))
        .layer(middleware::from_fn(assign_request_id))
}
