use chrono::NaiveDate;
use entity::{departments, employees, polyclinic, rooms};
use hospital_management_system::infra::config::AppConfig;

use migration::{Migrator, MigratorTrait};
use once_cell::sync::OnceCell;
use rand::{seq::IndexedRandom, Rng};
use sea_orm::{ActiveValue::Set, ConnectOptions, Database, DatabaseConnection, EntityTrait};
use std::time::Duration;
use tokio::sync::Mutex;

pub static DB_LOCK: OnceCell<Mutex<()>> = OnceCell::new();

pub fn init_db_lock() {
    DB_LOCK.get_or_init(|| Mutex::new(()));
}

pub struct TestContext {
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new() -> Self {
        let app_config =
            AppConfig::from_yaml("../application.yaml").expect("Failed to read app config");
        let test_url = app_config.database.test_url;

        let mut options = ConnectOptions::new(test_url);
        options
            .max_connections(10)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .sqlx_logging(true);

        let db = Database::connect(options)
            .await
            .expect("Failed to connect to the database");

        run_migration(&db).await;

        Self { db }
    }

    pub async fn reset(&self) {
        polyclinic::Entity::delete_many()
            .exec(&self.db)
            .await
            .expect("Failed to reset polyclinics");
        rooms::Entity::delete_many()
            .exec(&self.db)
            .await
            .expect("Failed to reset rooms");
        departments::Entity::delete_many()
            .exec(&self.db)
            .await
            .expect("Failed to reset departments");
    }

    pub async fn seed_departments(&self) {
        let department_list = vec![
            "Administrative",
            "Human Resource",
            "Finance",
            "IT",
            "Clinical",
            "IGD",
            "Emergency",
            "Procurement",
            "Nursing",
            "Laboratory",
            "Support",
        ];

        let department_models: Vec<_> = department_list
            .iter()
            .enumerate()
            .map(|(idx, &name)| {
                let code = format!("DPT{:02}", idx + 1);
                let department_category = name.replace(' ', "_").to_uppercase();
                departments::ActiveModel {
                    code: Set(code),
                    name: Set(name.to_string()),
                    department_category: Set(department_category),
                    description: Set(Some(format!(
                        "{} Department for regional hospital of xxx",
                        name
                    ))),
                    head_id: Set(None), // Assuming head_id can be null
                    status: Set("ACTIVE".to_string()),
                    ..Default::default()
                }
            })
            .collect();

        departments::Entity::insert_many(department_models)
            .exec(&self.db)
            .await
            .expect("Failed to seed departments");
    }

    pub async fn seed_employees(&self) {
        let mut rng = rand::rng();
        let employee_status = vec![
            "Permanent",
            "Contract",
            "Internship",
            "Resigned",
            "Terminated",
        ];
        let nums: Vec<u8> = (1..=11).collect();
        let mut employees_model = Vec::new();
        for i in 1..=150 {
            let code = *nums.choose(&mut rng).unwrap();
            let code_str = if code < 10 {
                format!("DPT0{}", code)
            } else {
                format!("DPT{}", code)
            };
            let birth_year = rng.random_range(1970..=1995);
            let birth_month = rng.random_range(1..=12);
            let hire_year = rng.random_range(2015..=2023);

            employees_model.push(employees::ActiveModel {
                full_name: Set(format!("Employee {}", i)),
                email: Set(format!("employee{}@test.com", i)),
                nip: Set(Some(format!(
                    "{:04}{:02}{:02}2000{:02}{}{:03}",
                    birth_year,
                    birth_month,
                    rng.random_range(1..=28),
                    birth_month,
                    rng.random_range(1..=2),
                    i
                ))),
                phone: Set(format!("+1234567890{}", i)),
                address: Set(format!("Address {}, City, Country", i)),
                department_code: Set(code_str), // Assuming 10 departments
                birth_date: Set(NaiveDate::from_ymd_opt(birth_year, 1, 1).unwrap()),
                employment_status: Set(employee_status.choose(&mut rng).unwrap().to_string()),
                hire_date: Set(NaiveDate::from_ymd_opt(hire_year, 1, 1).unwrap()),
                ..Default::default()
            });
        }
        employees::Entity::insert_many(employees_model)
            .exec(&self.db)
            .await
            .expect("Failed to seed employees");
    }

    pub async fn seed_rooms(&self, count: usize) {
        let rooms_data: Vec<_> = (1..=count)
            .map(|i| rooms::ActiveModel {
                code: Set(format!("RC{:02}", i)),
                status: Set("AVAILABLE".to_string()),
                room_type: Set("UNASSIGNED".to_string()),
                ..Default::default()
            })
            .collect();

        rooms::Entity::insert_many(rooms_data)
            .exec(&self.db)
            .await
            .expect("Failed to seed rooms");
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Here you can add any cleanup logic if needed
        // For example, closing the database connection
        println!("🔻 Connection closed and context dropped");
    }
}

pub async fn with_db_lock<F, Fut>(f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let lock = DB_LOCK.get().unwrap().lock().await;
    f().await;
    drop(lock);
}

pub async fn run_migration(db: &DatabaseConnection) {
    Migrator::up(db, None)
        .await
        .expect("Failed to perform migration");
}
