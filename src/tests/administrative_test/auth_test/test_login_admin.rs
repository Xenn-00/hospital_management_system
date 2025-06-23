use axum::{Router, body::Body, extract::Request, middleware, routing::post};
use http::StatusCode;
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::administrative::auth::auth_handler::login_handler,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::error_handler_layer::ErrorHandlingLayer,
    },
    state::AppState,
    tests::context::{LockKind, TestContext, with_lock},
};

#[tokio::test]
async fn test_login_admin() {
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

        TestContext::seed_departments(&ctx.db).await;
        TestContext::seed_role(&ctx.db).await;
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

        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .expect("Failed to collect body")
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("❗ Status: {status}, Body: {body_str}");

        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).expect("Failed to parse body bytes to json");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["message"], "Login successful");
        assert!(json["data"]["access_token"].is_string());

        TestContext::clean_records(&ctx.db).await;
    })
    .await
}
