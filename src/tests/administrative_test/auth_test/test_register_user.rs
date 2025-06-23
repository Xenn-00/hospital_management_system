use crate::{
    handlers::administrative::auth::auth_handler::register_handler,
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
        util::{admin_login, create_test_employee},
    },
};

use axum::{Router, body::Body, middleware, routing::post};
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::warn;

#[tokio::test]
async fn test_register_user() {
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

        let employee = create_test_employee(test_state.clone(), &access_token, &token_type).await;

        warn!("{:?}", employee["id"]);

        let employee_id = employee["id"]
            .as_i64()
            .expect("Failed to fetch employee id") as i32;

        let payload = json!({
            "employee_id": employee_id,
            "role": "ADMHR",
            "department_code": "DPT02"
        });

        let app = Router::new()
            .route("/api/v1/auth/protected/register", post(register_handler))
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

        let response = app
            .oneshot(
                Request::post("/api/v1/auth/protected/register")
                    .header("Content-Type", "application/json")
                    .header(
                        header::AUTHORIZATION,
                        format!("{token_type} {access_token}"),
                    )
                    .body(Body::from(payload.to_string()))
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to hit /api/v1/auth/protected/register");

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

        TestContext::clean_records(&ctx.db).await;
    })
    .await;
}
