use chrono::NaiveDate;
use entity::employees;
use log::info;
use rand::prelude::*;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter,
};

use crate::helpers::generate_employee;

pub async fn seeds_employees(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding employees...");
    let count = employees::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Employees already seeded. Skipping...");
        return Ok(());
    }

    let mut rng = rand::rng();

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
        hire_date: Set(NaiveDate::from_ymd_opt(rng.random_range(2015..=2023), 1, 1).unwrap()),
        employment_status: Set("Permanent".to_string()),
        birth_date: Set(NaiveDate::from_ymd_opt(
            rng.random_range(1970..=1995),
            rng.random_range(1..=12),
            rng.random_range(1..=28),
        )
        .unwrap()),
        gender: Set(1),
        created_by: Set(None),
        ..Default::default()
    };

    employees::Entity::insert(superadmin).exec(txn).await?;

    if let Some(superadmin_model) = employees::Entity::find()
        .filter(employees::Column::Email.eq("superadmin@admin.com".to_string()))
        .one(txn)
        .await?
    {
        let employees = generate_employee(100, superadmin_model.id);
        employees::Entity::insert_many(employees).exec(txn).await?;
        info!("✅ Employees seeded successfully.");
    };
    Ok(())
}
