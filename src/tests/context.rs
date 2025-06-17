use std::{collections::HashMap, sync::Arc, time::Duration as stdDuration};

use argon2::{
    Argon2, Params,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use aws_config::Region;
use aws_sdk_s3::{
    Client as S3Client,
    config::{Builder, Credentials, SharedCredentialsProvider},
};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::{Duration, NaiveDate, Utc};
use entity::{
    departments, employees, patients, patients_visit_intent, queue_ticket,
    users::{self, Role},
};

use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::prelude::*;
use redis::AsyncCommands;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectOptions, ConnectionTrait, Database,
    DatabaseBackend, DatabaseConnection, EntityTrait, QueryFilter, Statement, TransactionTrait,
};

use serde_json::json;
use tokio::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{
    dtos::triage::create_triage_request::CreateTriageRequest, infra::config::AppConfig,
    utils::jwt::JwtKeys,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LockKind {
    Triage,
    Auth,
    Global,
}

static LOCKS: Lazy<HashMap<LockKind, Arc<Mutex<()>>>> = Lazy::new(|| {
    use LockKind::*;
    HashMap::from([
        (Triage, Arc::new(Mutex::new(()))),
        (Auth, Arc::new(Mutex::new(()))),
        (Global, Arc::new(Mutex::new(()))),
    ])
});

pub async fn with_lock<F, Fut>(kind: LockKind, f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let guard = LOCKS.get(&kind).expect("Lock not initialized").lock().await;
    f().await;
    drop(guard);
}

pub struct TestContext {
    pub db: DatabaseConnection,
    pub redis: Pool<RedisConnectionManager>,
    pub s3: S3Client,
    pub jwt_keys: JwtKeys,
}

impl TestContext {
    pub async fn new() -> Self {
        let config = AppConfig::from_yaml("application.yaml").expect("Failed to load config");

        let mut db_opts = ConnectOptions::new(config.database.test_url.clone());

        db_opts
            .max_connections(5)
            .connect_timeout(stdDuration::from_secs(10))
            .sqlx_logging(false);

        let db = Database::connect(db_opts)
            .await
            .expect("Failed to connect to test database");

        let redis_manager = RedisConnectionManager::new(config.redis.upstash_redis_url)
            .expect("Failed to connect to redis server");

        let redis_pool = Pool::builder()
            .max_size(10)
            .min_idle(Some(5))
            .idle_timeout(Some(stdDuration::from_secs(15)))
            .max_lifetime(Some(stdDuration::from_secs(60)))
            .build(redis_manager)
            .await
            .expect("Failed to build Redis pool");

        let s3_creds = Credentials::new(
            &config.s3.s3_access_key,
            &config.s3.s3_secret_key,
            None,
            None,
            "static",
        );

        let shared_creds = SharedCredentialsProvider::new(s3_creds);

        let conf = Builder::new()
            .region(Region::new(config.s3.s3_region.clone()))
            .credentials_provider(shared_creds)
            .endpoint_url(&config.s3.s3_url)
            .force_path_style(true)
            .build();

        let s3_client = S3Client::from_conf(conf);

        let private_key = std::fs::read("private_key.pem").expect("Failed to fetch private key");
        let public_key = std::fs::read("public_key.pem").expect("Failed to fetch public key");

        Self {
            db,
            redis: redis_pool,
            s3: s3_client,
            jwt_keys: JwtKeys {
                encoding: Arc::new(
                    EncodingKey::from_rsa_pem(&private_key).expect("Failed to set encoding key"),
                ),
                decoding: Arc::new(
                    DecodingKey::from_rsa_pem(&public_key).expect("Failed to set decoding key"),
                ),
            },
        }
    }

    pub async fn clean_records(conn: &DatabaseConnection) {
        let stmt = Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
                TRUNCATE TABLE 
                    patients, 
                    patients_visit_intent, 
                    queue_ticket, 
                    referral_documents, 
                    departments 
                RESTART IDENTITY CASCADE;
            "#,
        );
        conn.execute(stmt)
            .await
            .expect("Failed to execute statement");
    }

    pub async fn seed_departments(conn: &DatabaseConnection) {
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

        let department_models: Vec<departments::ActiveModel> = department_list
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
            .on_conflict_do_nothing()
            .exec(conn)
            .await
            .expect("Failed to seed departments");
    }

    pub async fn seed_employee_and_user(conn: &DatabaseConnection) {
        let mut rng = rand::rng();

        // 1. create employee

        let superadmin = employees::ActiveModel {
            full_name: Set(format!("Superadmin Employee")),
            email: Set(format!("superadmin@admin.com")),
            nip: Set(Some(format!(
                "{:04}{:02}{:02}2000{:02}{}{:03}",
                rng.random_range(1970..=1995),
                rng.random_range(1..=12),
                rng.random_range(1..=28),
                rng.random_range(1..=12),
                rng.random_range(1..=2),
                1
            ))),

            phone: Set(format!("+77777777777")),
            address: Set(format!("Address for superadmin")),
            department_code: Set("DPT01".to_string()),
            hire_date: Set(NaiveDate::from_ymd_opt(rng.random_range(2015..=2023), 1, 1)
                .expect("Failed to format date")),
            employment_status: Set("Permanent".to_string()),
            birth_date: Set(NaiveDate::from_ymd_opt(
                rng.random_range(1970..=1995),
                rng.random_range(1..=12),
                rng.random_range(1..=28),
            )
            .expect("Failed to format date")),
            gender: Set(1),
            created_by: Set(None),
            ..Default::default()
        };

        employees::Entity::insert(superadmin)
            .exec(conn)
            .await
            .expect("Failed to insert employee");

        // 2. create user

        let now = Utc::now();

        if let Some(emp) = employees::Entity::find()
            .filter(employees::Column::Email.eq("superadmin@admin.com"))
            .one(conn)
            .await
            .expect("Failed to get employee")
        {
            let password_raw = format!("test_password");
            let salt = SaltString::generate(&mut OsRng);

            let argon2 = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                Params::new(8, 1, 1, None).expect("Failed to initialize argon"),
            );

            let password_hash = argon2
                .hash_password(password_raw.as_bytes(), &salt)
                .expect("Failed to hash password")
                .to_string();

            let user = users::ActiveModel {
                employee_id: Set(emp.id),
                password: Set(password_hash),
                username: Set(format!("superadmin")),
                role: Set(Role::Superadmin),
                last_login: Set(Some(
                    (now - Duration::days(rng.random_range(1..=30))).naive_utc(),
                )),
                is_active: Set(true),
                ..Default::default()
            };

            users::Entity::insert(user)
                .exec(conn)
                .await
                .expect("Failed to insert test user");
        }
    }

    pub async fn seed_patient(conn: &DatabaseConnection, visit_type: &str) {
        let txn = conn.begin().await.expect("Failed to create transaction");
        let payload: CreateTriageRequest = serde_json::from_value(json!({
            "name": "test test",
            "date_of_birth": "2004-09-12",
            "national_id": "3326080812331122",
            "gender": "Male",
            "emergency_contact_name": "Test test",
            "emergency_contact_phone": "+8181234569999",
            "emergency_contact_relationship": "Father",
            "blood_type": "o+",
            "known_allergies": "peanut",
            "visit_type": visit_type
        }))
        .expect("Failed to deserialize payload into CreateTriageRequest");

        let patient = patients::ActiveModel {
            name: Set(payload.name.clone()),
            date_of_birth: Set(payload.date_of_birth),
            national_id: Set(payload.national_id.clone()),
            bpjs_number: Set(payload.bpjs_number.clone()),
            gender: Set(payload.gender.to_string()),
            emergency_contact_name: Set(payload.emergency_contact_name.clone()),
            emergency_contact_phone: Set(payload.emergency_contact_phone.clone()),
            emergency_contact_relationship: Set(payload.emergency_contact_relationship.clone()),
            blood_type: Set(payload.blood_type.to_string()),
            known_allergies: Set(Some(payload.known_allergies.clone().unwrap_or_default())),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .expect("Failed to insert patient");

        let visit_intent = patients_visit_intent::ActiveModel {
            patient_id: Set(patient.id),
            visit_type: Set(payload.visit_type.to_string().to_uppercase()),
            status: Set("WAITING".into()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .expect("Failed to insert patient visit intent");

        queue_ticket::ActiveModel {
            visit_intent_id: Set(visit_intent.id),
            queue_number: Set(1),
            queue_type: Set(visit_type.to_string().to_uppercase()),
            status: Set("WAITING".into()),

            ..Default::default()
        }
        .insert(&txn)
        .await
        .expect("Failed to insert queue ticket");

        txn.commit().await.expect("Failed to commit transaction");
    }

    pub async fn cleanup_redis_keys(
        mut conn: bb8::PooledConnection<'_, RedisConnectionManager>,
        keys: &str,
    ) {
        let _: () = conn.del(keys).await.expect("Failed to delete cache");
    }
}
