use std::time::Duration;

use hospital_management_system::infra::config::AppConfig;
use sea_orm::{ConnectOptions, Database};
use seeder::seed_all;

use log::info;

#[tokio::main]
async fn main() {
    log4rs::init_file("seeder/seedLog.yaml", Default::default()).unwrap();
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

    info!("🟢 Starting database seeding...");
    seed_all(&db).await.expect("🔴 failed to seed the database");
    info!("🟢 Database seeding done!");
}
