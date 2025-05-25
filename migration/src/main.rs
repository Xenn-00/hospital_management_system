use std::time::Duration;

use hospital_management_system::infra::config::AppConfig;
use migration::{
    sea_orm::{ConnectOptions, Database},
    Migrator,
};
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let app_config = AppConfig::from_yaml("application.yaml").unwrap();

    let direct_url = app_config.database.direct_url;

    let mut options = ConnectOptions::new(direct_url);
    options
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);

    let db = Database::connect(options).await.unwrap();

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migration");
    println!("Migration completed successfully");
}
