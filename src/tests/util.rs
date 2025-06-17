use axum::{Router, body::Body, middleware, routing::post};
use entity::{patients, patients_visit_intent, queue_ticket};
use http::Request;
use http_body_util::BodyExt;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde_json::json;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    dtos::triage::create_triage_request::CreateTriageRequest,
    error_handling::app_error::AppError,
    handlers::administrative::auth::auth_handler::login_handler,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::error_handler_layer::ErrorHandlingLayer,
    },
    state::AppState,
    tests::context::TestContext,
};

pub async fn call_patient(conn: &DatabaseConnection, visit_type: &str) {
    let txn = conn.begin().await.expect("Failed to create transaction");
    let payload: CreateTriageRequest = serde_json::from_value(json!({
        "name": "test test",
        "date_of_birth": "2004-09-12",
        "national_id": "3326080812331122",
        "gender": "Male",
        "emergency_contact_name": "Test test",
        "emergency_contact_phone": "+8181234569999",
        "emergency_contact_relationship": "Father",
        "blood_type": "o+",
        "known_allergies": "peanut",
        "visit_type": visit_type
    }))
    .expect("Failed to deserialize payload into CreateTriageRequest");

    let patient = patients::ActiveModel {
        name: Set(payload.name.clone()),
        date_of_birth: Set(payload.date_of_birth),
        national_id: Set(payload.national_id.clone()),
        bpjs_number: Set(payload.bpjs_number.clone()),
        gender: Set(payload.gender.to_string()),
        emergency_contact_name: Set(payload.emergency_contact_name.clone()),
        emergency_contact_phone: Set(payload.emergency_contact_phone.clone()),
        emergency_contact_relationship: Set(payload.emergency_contact_relationship.clone()),
        blood_type: Set(payload.blood_type.to_string()),
        known_allergies: Set(Some(payload.known_allergies.clone().unwrap_or_default())),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to insert patient");

    let visit_intent = patients_visit_intent::ActiveModel {
        patient_id: Set(patient.id),
        visit_type: Set(payload.visit_type.to_string().to_uppercase()),
        status: Set("CALLED".into()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to insert patient visit intent");

    queue_ticket::ActiveModel {
        visit_intent_id: Set(visit_intent.id),
        queue_number: Set(1),
        queue_type: Set(visit_type.to_string().to_uppercase()),
        status: Set("CALLED".into()),

        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to insert queue ticket");

    txn.commit().await.expect("Failed to commit transaction");
}

pub async fn get_patient(
    visit_type: &str,
    test_state: AppState,
) -> Result<queue_ticket::Model, AppError> {
    if let Some(existing) = queue_ticket::Entity::find()
        .filter(queue_ticket::Column::QueueNumber.eq(1))
        .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string().to_uppercase()))
        .one(&test_state.db)
        .await?
    {
        return Ok(existing);
    }

    Err(AppError::NotFound(format!(
        "Queue {} is not found with number {}",
        visit_type, 1
    )))
}

pub async fn admin_login(test_state: AppState) -> serde_json::Value {
    let ctx = TestContext::new().await;

    TestContext::seed_departments(&ctx.db).await;
    TestContext::seed_employee_and_user(&ctx.db).await;

    let app = Router::new()
        .route("/api/v1/auth/login", post(login_handler))
        .layer(TraceLayer::new_for_http())
        .layer(ErrorHandlingLayer)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(middleware::from_fn(assign_request_id))
        .with_state(test_state);

    let payload = json!({
        "username": "superadmin",
        "password": "test_password"
    });

    let response = app
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("Failed to create login request"),
        )
        .await
        .expect("Failed to fetch /api/v1/auth/login");

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("Failed to collect body")
        .to_bytes();

    let json: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse body bytes to json");

    return json["data"].clone();
}
