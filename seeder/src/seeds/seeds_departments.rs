use entity::departments;
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_department;

pub async fn seed_departments(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    // Insert sample data into the Departments table
    info!("🚀 Seeding departments...");
    let count = departments::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Departments already seeded. Skipping...");
        return Ok(());
    }

    let department_list = vec![
        "Administrative",
        "Human Resource",
        "Finance",
        "IT",
        "Clinical",
        "Emergency",
        "Procurement",
        "Nursing",
        "Laboratory",
        "Support",
    ];

    let departments = generate_department(department_list);

    departments::Entity::insert_many(departments)
        .exec(txn)
        .await?;

    info!("✅ Departments seeded successfully.");

    Ok(())
}
