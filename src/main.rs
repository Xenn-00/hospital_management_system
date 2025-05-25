use axum::{Router, middleware};
use hospital_management_system::{
    infra::config::AppConfig,
    middleware::request_middleware::assign_request_id,
    router::triage_route::triage_routes,
    state::{self, init_database_connection, init_redis_pool, init_s3_client},
};
use log::info;

use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let app_config =
        AppConfig::from_yaml("application.yaml").expect("Failed to load application.yaml");

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

    let app_state = state::AppState {
        db,
        redis: redis_pool,
        s3,
    };

    let app = Router::new()
        .nest("/api/v1", triage_routes(app_state.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(assign_request_id)))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024));

    let listener = TcpListener::bind(format!("{}:{}", app_config.app.host, app_config.app.port))
        .await
        .expect("Failed to bind to address");
    info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    // wait for CTRL+C
    signal::ctrl_c()
        .await
        .expect("Failed to listen for shutdown signal");
    info!("Shutdown signal received, exiting...");
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC OCCURRED: {:?}", panic_info);
    }));
}
