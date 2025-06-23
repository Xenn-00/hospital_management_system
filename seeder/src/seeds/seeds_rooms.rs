use entity::rooms;
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_room;

pub async fn seeds_rooms(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding rooms...");
    let count = rooms::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Rooms already seeded. Skipping...");
        return Ok(());
    }
    let rooms = generate_room(36);
    rooms::Entity::insert_many(rooms).exec(txn).await?;
    info!("✅ Rooms seeded successfully.");
    Ok(())
}
