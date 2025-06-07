use entity::{employees, user};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_users;

pub async fn seeds_users(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding users...");
    let count = user::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  User already seeded. Skipping...");
        return Ok(());
    }

    let employee = employees::Entity::find().all(txn).await?;
    let employee_ids_and_dept: Vec<(i32, String)> = employee
        .into_iter()
        .map(|emp| (emp.id, emp.department_code))
        .collect();

    let user_models = generate_users(employee_ids_and_dept);

    user::Entity::insert_many(user_models).exec(txn).await?;
    info!("✅ Users seeded successfully.");

    Ok(())
}
