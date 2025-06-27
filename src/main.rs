use axum::{Router, middleware};
use hospital_management_system::{
    infra::config::AppConfig,
    middleware::{
        fn_middleware::request_middleware::assign_request_id,
        layer_middleware::{
            authenticate_layer::JwtAuthLayer, error_handler_layer::ErrorHandlingLayer,
        },
    },
    router::{
        administrative::{
            auth_route::{auth_routes_protected, auth_routes_public},
            employment_route::employment_routes,
        },
        medical::polyclinic_route::polyclinic_routes,
        patients::triage_route::triage_routes,
    },
    state::{AppState, init_database_connection, init_redis_pool, init_s3_client, load_jwt_keys},
    utils::worker::{
        cron_register_setup_cleanup::cron_register_setup_cleanup, worker_send_otp::worker_send_otp,
    },
};

use std::{net::ToSocketAddrs, sync::Arc, time::Duration};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .try_init();
    let app_config = AppConfig::from_yaml("application.yaml")
        .await
        .expect("Failed to load application.yaml");

    let jwt_keys = load_jwt_keys().await.expect("Failed to fetch jwt keys");

    let twilio = app_config.twilio;

    // Parallel initialization
    let (db, redis_pool, s3) = tokio::join!(
        init_database_connection(&app_config.database.url),
        init_redis_pool(&app_config.redis.docker_redis_url),
        init_s3_client(&app_config.s3)
    );

    tracing::info!("Connected to DB, Redis, and S3 successfully");

    // Crons
    tokio::spawn(cron_register_setup_cleanup(db.clone(), redis_pool.clone()));
    tokio::spawn(worker_send_otp(
        twilio.clone(),
        app_config.redis.docker_redis_url.clone(),
    ));

    let app_state = Arc::new(AppState {
        db: db.into(),
        redis: redis_pool.into(),
        s3: s3.into(),
        jwt_keys: jwt_keys.into(),
        twilio: twilio.into(),
    });

    let app = build_router(app_state);

    let bind_address = format!("{}:{}", app_config.app.host, app_config.app.port);
    if bind_address.to_socket_addrs().is_err() {
        panic!("Invalid host or port: {}", bind_address);
    }

    let listener = TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", bind_address));

    tracing::info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn build_router(app_state: Arc<AppState>) -> Router {
    let authed_layer = JwtAuthLayer {
        app_state: (*app_state).clone(),
    };

    Router::new()
        .nest("/api/v1/auth/public", auth_routes_public())
        .nest(
            "/api/v1/triage",
            triage_routes(app_state.clone().into()).layer(authed_layer.clone()),
        )
        .nest(
            "/api/v1/auth/protected",
            auth_routes_protected(app_state.clone().into()).layer(authed_layer.clone()),
        )
        .nest(
            "/api/v1/employee",
            employment_routes(app_state.clone().into()).layer(authed_layer.clone()),
        )
        .nest(
            "/api/v1/medical",
            polyclinic_routes(app_state.clone().into()).layer(authed_layer),
        )
        .layer(middleware::from_fn(assign_request_id))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10 MB
                .layer(TimeoutLayer::new(Duration::from_secs(15))) // Timeout protection
                .layer(ErrorHandlingLayer),
        )
        .layer(TraceLayer::new_for_http()) // Logging each request (outermost)
        .with_state((*app_state).clone())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, exiting...");
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC OCCURRED: {:?}", panic_info);
    }));
}
