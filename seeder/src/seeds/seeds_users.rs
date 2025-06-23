use std::collections::HashMap;

use entity::{employees, role, users};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_users;

pub async fn seeds_users(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding users...");
    let count = users::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  User already seeded. Skipping...");
        return Ok(());
    }

    let employees = employees::Entity::find().all(txn).await?;

    let roles = role::Entity::find().all(txn).await?;

    let role_code_to_id: HashMap<String, i32> = roles.into_iter().map(|r| (r.code, r.id)).collect();

    let user_models = generate_users(employees, role_code_to_id);

    users::Entity::insert_many(user_models).exec(txn).await?;
    info!("✅ Users seeded successfully.");

    Ok(())
}
