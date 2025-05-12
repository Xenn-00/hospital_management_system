use std::time::Duration;

use axum::Router;
use hospital_management_system::{
    infra::config::AppConfig, router::triage_route::triage_routes, state,
};
use log::info;
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;

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

    info!("Connected to the database successfully");

    let app_state = state::AppState { db };

    let app = Router::new().nest("/api/v1", triage_routes(app_state));

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
