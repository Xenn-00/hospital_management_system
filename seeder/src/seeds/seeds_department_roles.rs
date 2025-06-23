use std::collections::HashMap;

use entity::{department_roles, role};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_department_roles;

pub async fn seeds_department_roles(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding department roles...");
    let count = department_roles::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️ Deaprtment roles already seeded. Skipping...");
        return Ok(());
    }

    let roles = role::Entity::find().all(txn).await?;

    let role_code_to_id: HashMap<String, i32> = roles.into_iter().map(|r| (r.code, r.id)).collect();

    let models = generate_department_roles(role_code_to_id);

    department_roles::Entity::insert_many(models)
        .exec(txn)
        .await?;
    info!("✅ Department roles seeded successfully.");
    Ok(())
}
