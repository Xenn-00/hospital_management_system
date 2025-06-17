use axum::{Router, body::Body, extract::Request, middleware, routing::post};
use http::{StatusCode, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use tracing_subscriber;

use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::triage::triage_handler::triage_patient,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::{
            authenticate_layer::JwtAuthLayer, error_handler_layer::ErrorHandlingLayer,
        },
        rbac_middleware::rbac_staff_only::rbac_staff_only,
    },
    state::AppState,
    tests::{
        context::{LockKind, TestContext, with_lock},
        util::admin_login,
    },
};

#[tokio::test]
async fn test_perfom_triage() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
    with_lock(LockKind::Global, || async {
        let ctx = TestContext::new().await;
        TestContext::clean_records(&ctx.db).await;

        let test_state = AppState {
            db: ctx.db.clone(),
            redis: ctx.redis.clone(),
            s3: ctx.s3.clone(),
            jwt_keys: ctx.jwt_keys.clone(),
        };

        let login = admin_login(test_state.clone()).await;
        let access_token = login["access_token"]
            .as_str()
            .expect("Failed to fetch access_token");

        let token_type = login["token_type"]
            .as_str()
            .expect("Failed to fetch token_type");

        let app = Router::new()
            .route("/api/v1/triage", post(triage_patient))
            .layer(TraceLayer::new_for_http())
            .layer(ErrorHandlingLayer)
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(middleware::from_fn_with_state(
                test_state.clone().into(),
                rbac_staff_only,
            ))
            .layer(JwtAuthLayer {
                app_state: test_state.clone(),
            })
            .layer(middleware::from_fn(assign_request_id))
            .with_state(test_state);

        let payload = json!({
            "name": "test test",
            "date_of_birth": "2004-09-12",
            "national_id": "3326080812331122",
            "gender": "Male",
            "emergency_contact_name": "Test test",
            "emergency_contact_phone": "+8181234569999",
            "emergency_contact_relationship": "Father",
            "blood_type": "o+",
            "known_allergies": "peanut",
            "visit_type": "bpjs"
        });

        let response = app
            .oneshot(
                Request::post("/api/v1/triage")
                    .header("Content-Type", "application/json")
                    .header(
                        header::AUTHORIZATION,
                        format!("{token_type} {access_token}"),
                    )
                    .body(Body::from(payload.to_string()))
                    .expect("Failed to create triage request"),
            )
            .await
            .expect("Failed to fetch /api/v1/triage");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("Failed to collect body response")
            .to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed parsing body bytes to json");

        assert_eq!(json["message"], "Triage successful");
        assert!(json["data"]["patient_id"].is_number());

        TestContext::clean_records(&ctx.db).await;
    })
    .await;
}
