use entity::{nurses, nurses_polyclinic_assignments};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_nurses_polyclinic_assignments;

pub async fn seeds_nurse_polyclinic_assignments(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding nurse polyclinic assignments...");
    let count = entity::nurses_polyclinic_assignments::Entity::find()
        .count(txn)
        .await?;
    if count > 0 {
        info!("⚠️  Doctors already seeded. Skipping...");
        return Ok(());
    }

    let nurses = nurses::Entity::find().all(txn).await?;
    let nurse_ids_and_poly_ids: Vec<(i32, i32)> = nurses
        .into_iter()
        .map(|n| (n.id, n.polyclinic_id))
        .collect();

    let models = generate_nurses_polyclinic_assignments(nurse_ids_and_poly_ids);

    nurses_polyclinic_assignments::Entity::insert_many(models)
        .exec(txn)
        .await?;
    info!("✅ Nurse polyclinic assignments seeded successfully.");
    Ok(())
}
