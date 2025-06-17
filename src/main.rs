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
        administrative::{auth_route::auth_routes, employment_route::employment_routes},
        triage_route::triage_routes,
    },
    state::{AppState, init_database_connection, init_redis_pool, init_s3_client, load_jwt_keys},
};
use log::info;

use std::net::ToSocketAddrs;
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let app_config =
        AppConfig::from_yaml("application.yaml").expect("Failed to load application.yaml");

    let jwt_keys = load_jwt_keys().expect("Failed to fetch jwt keys");

    // database config
    let url = app_config.database.url;
    let db = init_database_connection(&url).await;

    // redis config
    let redis_url = app_config.redis.upstash_redis_url;
    let redis_pool = init_redis_pool(&redis_url).await;

    // s3 config
    let s3_config = app_config.s3;
    let s3 = init_s3_client(&s3_config).await;

    info!("Connected to the database successfully");
    info!("Connected to the redis successfully");
    info!("Connected to the s3 successfully");

    let app_state = AppState {
        db,
        redis: redis_pool,
        s3,
        jwt_keys,
    };

    let app = build_router(app_state);

    let bind_address = format!("{}:{}", app_config.app.host, app_config.app.port);
    if bind_address.to_socket_addrs().is_err() {
        panic!("Invalid host or port: {}", bind_address);
    }

    let listener = TcpListener::bind(&bind_address)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {}", bind_address));

    info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn build_router(app_state: AppState) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(auth_routes(app_state.clone().into()))
                // Authenticated + staff role access only
                .nest(
                    "/triage",
                    triage_routes(app_state.clone().into()).layer(JwtAuthLayer {
                        app_state: app_state.clone(),
                    }),
                )
                .nest(
                    "/employee",
                    employment_routes(app_state.clone().into()).layer(JwtAuthLayer {
                        app_state: app_state.clone(),
                    }),
                ),
        )
        .layer(ErrorHandlingLayer)
        .layer(ServiceBuilder::new().layer(middleware::from_fn(assign_request_id)))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .with_state(app_state)
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

    info!("Shutdown signal received, exiting...");
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC OCCURRED: {:?}", panic_info);
    }));
}
