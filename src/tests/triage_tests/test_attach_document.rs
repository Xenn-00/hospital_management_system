use std::fs;

use axum::{Router, body::Body, middleware, routing::post};
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;

use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::triage::triage_handler::triage_referral_document_upload,
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
        util::{admin_login, get_patient},
    },
};

#[tokio::test]
async fn test_attach_document() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
    with_lock(LockKind::Global, || async {
        let ctx = TestContext::new().await;
        let visit_type = "bpjs";
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

        let redis_conn = ctx.redis.get().await.expect("Failed to get connection");

        let patient = get_patient(visit_type, test_state.clone())
            .await
            .expect("Failed to get patient");

        let app = Router::new()
            .route(
                "/api/v1/triage/patient/{patient_id}/{visit_id}/upload",
                post(triage_referral_document_upload),
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

        let file_bytes = fs::read("src/tests/triage_tests/asset/asset-1.png")
            .expect("Failed to read file or path is unreachable");

        let boundary = "TESTBOUNDARY";
        let mut multipart_body: Vec<u8> = Vec::new();
        multipart_body.extend(
            format!(
                "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"asset-1.png\"\r\n\
        Content-Type: image/png\r\n\r\n"
            )
            .as_bytes(),
        );

        multipart_body.extend(&file_bytes);
        multipart_body.extend(format!("\r\n--{boundary}--\r\n").as_bytes());

        let body = Body::from(multipart_body);

        let response = app
            .oneshot(
                Request::post(format!(
                    "/api/v1/triage/patient/{}/{}/upload",
                    patient.id, patient.visit_intent_id
                ))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .header(
                    header::AUTHORIZATION,
                    format!("{token_type} {access_token}"),
                )
                .body(body)
                .expect("Failed to create request"),
            )
            .await
            .expect("Failed to get response");

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

        assert_eq!(json["message"], "file upload complete");

        TestContext::clean_records(&ctx.db).await;
        TestContext::cleanup_redis_keys(redis_conn, "triage:queue:bpjs").await;
    })
    .await
}
