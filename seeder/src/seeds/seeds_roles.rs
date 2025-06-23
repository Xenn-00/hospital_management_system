use entity::role;
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_roles;

pub async fn seeds_roles(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding roles...");
    let count = role::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️ Roles already seeded. Skipping...");
        return Ok(());
    }
    let roles = generate_roles();
    role::Entity::insert_many(roles).exec(txn).await?;
    info!("✅ Roles seeded successfully.");
    Ok(())
}
