use entity::employees;
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_employee;

pub async fn seeds_employees(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding employees...");
    let count = employees::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Employees already seeded. Skipping...");
        return Ok(());
    }

    let employees = generate_employee(100);
    employees::Entity::insert_many(employees).exec(txn).await?;
    info!("✅ Employees seeded successfully.");
    Ok(())
}
