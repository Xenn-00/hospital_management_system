use std::time::Duration;

use axum::{Router, middleware};
use hospital_management_system::{
    infra::config::AppConfig,
    middleware::request_middleware::assign_request_id,
    router::triage_route::triage_routes,
    state::{self, init_redis_pool},
};
use log::info;
use sea_orm::{ConnectOptions, Database};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let app_config = AppConfig::from_yaml("application.yaml").unwrap();

    let url = app_config.database.url;
    let mut options = ConnectOptions::new(&url);

    options
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    let db = Database::connect(options)
        .await
        .expect("Failed to connect to the database");

    let redis_url = app_config.redis.upstash_redis_url;
    let redis_pool = init_redis_pool(&redis_url).await;

    info!("Connected to the database successfully");
    info!("Connected to the redis successfully");

    let app_state = state::AppState {
        db,
        redis: redis_pool,
    };

    let app = Router::new()
        .nest("/api/v1", triage_routes(app_state))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(assign_request_id)));

    let listener = TcpListener::bind(format!("{}:{}", app_config.app.host, app_config.app.port))
        .await
        .unwrap();
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
}
