use entity::nurses;
use log::info;
use sea_orm::{ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait, QueryFilter};

use crate::helpers::generate_nurses;

pub async fn seeds_nurses(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding nurses...");
    let count = entity::nurses::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Nurses already seeded. Skipping...");
        return Ok(());
    }

    let employees = entity::employees::Entity::find()
        .filter(entity::employees::Column::EmploymentStatus.ne("TERMINATED".to_string()))
        .filter(entity::employees::Column::EmploymentStatus.ne("RESIGNED".to_string()))
        .filter(entity::employees::Column::DepartmentCode.eq("DPT05".to_string()))
        .left_join(entity::doctors::Entity)
        .filter(entity::doctors::Column::EmployeeId.is_null())
        .all(txn)
        .await?;

    if employees.is_empty() {
        info!("⚠️  No employees found...");
        return Err(DbErr::Custom(
            "No employees found to seed doctors".to_string(),
        ));
    }

    let filled_polyclinics = entity::doctors::Entity::find().all(txn).await?;

    let filled_polyclinics_ids: Vec<i32> = filled_polyclinics
        .into_iter()
        .map(|poly| poly.polyclinic_id)
        .collect();

    let employee_ids: Vec<i32> = employees.into_iter().map(|emp| emp.id).collect();

    let nurse_models = generate_nurses(
        employee_ids.len() as i32,
        employee_ids,
        filled_polyclinics_ids,
    );

    nurses::Entity::insert_many(nurse_models).exec(txn).await?;
    info!("✅ Nurses seeded successfully.");

    Ok(())
}
