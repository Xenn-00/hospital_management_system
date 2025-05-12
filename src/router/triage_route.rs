use axum::middleware;
use axum::{Router, routing::post};

use crate::handlers::triage::triage_handler::triage_patient;
use crate::middleware::request_middleware::assign_request_id;
use crate::state::AppState;

pub fn triage_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/triage", post(triage_patient))
        .layer(middleware::from_fn(assign_request_id))
        .with_state(app_state)
}
