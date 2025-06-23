use axum::{Router, body::Body, middleware, routing::patch};
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use redis::AsyncCommands;
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::triage::triage_handler::triage_patient_cancel,
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
async fn test_patient_cancel() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
    with_lock(LockKind::Global, || async {
        let ctx = TestContext::new().await;
        let visit_type = "common";
        TestContext::clean_records(&ctx.db).await;

        TestContext::seed_patient(&ctx.db, visit_type).await;

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
            .route(
                "/api/v1/triage/queue/{visit_type}/{queue_number}/cancel",
                patch(triage_patient_cancel),
            )
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

        let cache_key = format!("cancel:queue:{visit_type}:id:1");
        let mut redis_conn = ctx.redis.get().await.expect("Failed to get connection");
        let _: () = redis_conn
            .del(&cache_key)
            .await
            .expect("Failed to delete cache");

        let response = app
            .oneshot(
                Request::patch(format!("/api/v1/triage/queue/{visit_type}/1/cancel"))
                    .header(
                        header::AUTHORIZATION,
                        format!("{token_type} {access_token}"),
                    )
                    .body(Body::empty())
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to get /api/v1/triage/queue/{visit_type}/{queue_number}/cancel");

        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .expect("Failed to fetch response into body")
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);

        println!("❗ Status: {status}, Body: {body_str}");

        assert_eq!(status, StatusCode::OK);

        let json: serde_json::Value = serde_json::from_slice(&body_bytes)
            .expect("Failed to convert body bytes into json value");

        assert_eq!(json["message"], "Cancel triage patient complete");
        assert_eq!(json["data"]["previous_status"], "WAITING");
        assert_eq!(json["data"]["queue_type"], visit_type.to_uppercase());

        TestContext::clean_records(&ctx.db).await;
        TestContext::cleanup_redis_keys(redis_conn, &cache_key).await;
    })
    .await
}
