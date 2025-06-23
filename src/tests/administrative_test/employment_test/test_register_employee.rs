use crate::{
    handlers::administrative::employment::employee_handler::register_employee,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::{
            authenticate_layer::JwtAuthLayer, error_handler_layer::ErrorHandlingLayer,
        },
        rbac_middleware::rbac_superadmin_only::rbac_superadmin_only,
    },
    state::AppState,
    tests::{
        context::{LockKind, TestContext, with_lock},
        util::admin_login,
    },
};
use axum::{Router, body::Body, extract::Request, middleware, routing::post};
use http::{StatusCode, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

#[tokio::test]
async fn test_employee_register() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
    with_lock(LockKind::Global, || async {
        let ctx = TestContext::new().await;
        TestContext::clean_records(&ctx.db).await;

        let test_state = AppState {
            db: ctx.db.clone().into(),
            redis: ctx.redis.clone().into(),
            s3: ctx.s3.clone().into(),
            jwt_keys: ctx.jwt_keys.clone().into(),
            twilio: ctx.twilio.into(),
        };

        let login = admin_login(test_state.clone()).await;
        let access_token = login["access_token"]
            .as_str()
            .expect("Failed to fetch access_token");

        let token_type = login["token_type"]
            .as_str()
            .expect("Failed to fetch token_type");

        let app = Router::new()
            .route("/api/v1/employee/register", post(register_employee))
            .layer(TraceLayer::new_for_http())
            .layer(ErrorHandlingLayer)
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(middleware::from_fn_with_state(
                test_state.clone(),
                rbac_superadmin_only,
            ))
            .layer(JwtAuthLayer {
                app_state: test_state.clone(),
            })
            .layer(middleware::from_fn(assign_request_id))
            .with_state(test_state);

        let payload = json!({
            "full_name": "Test Test",
            "gender": "Male",
            "nip": "197812312024011001",
            "email": "test.test@example.com",
            "phone": "+6281234567899",
            "birth_date": "1978-12-31",
            "hire_date": "2024-01-01",
            "address": "Testing Ave No. 99, Test",
            "employement_status": "Permanent",
            "department_code": "DPT02"
        });

        let response = app
            .oneshot(
                Request::post("/api/v1/employee/register")
                    .header("Content-Type", "application/json")
                    .header(
                        header::AUTHORIZATION,
                        format!("{token_type} {access_token}"),
                    )
                    .body(Body::from(payload.to_string()))
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to hit /api/v1/employee/register");

        assert_eq!(response.status(), StatusCode::OK);

        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .expect("Failed to fetch response into body")
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("❗ Status: {status}, Body: {body_str}");

        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).expect("Failed to parse value");

        assert_eq!(json["message"], "Register employee success");
        assert!(json["data"]["id"].is_number());

        TestContext::clean_records(&ctx.db).await;
    })
    .await;
}
