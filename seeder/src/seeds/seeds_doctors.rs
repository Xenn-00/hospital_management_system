use entity::doctors;
use log::info;
use sea_orm::{ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait, QueryFilter};

use crate::helpers::generate_doctor;

pub async fn seeds_doctors(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding doctors...");
    let count = entity::doctors::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Doctors already seeded. Skipping...");
        return Ok(());
    }
    let employees = entity::employees::Entity::find()
        .filter(entity::employees::Column::EmploymentStatus.ne("TERMINATED".to_string()))
        .filter(entity::employees::Column::EmploymentStatus.ne("RESIGNED".to_string()))
        .filter(entity::employees::Column::DepartmentCode.eq("DPT05".to_string()))
        .all(txn)
        .await?;
    if employees.is_empty() {
        info!("⚠️  No employees found...");
        return Err(DbErr::Custom(
            "No employees found to seed doctors".to_string(),
        ));
    }

    let polyclinics = entity::polyclinic::Entity::find().all(txn).await?;

    let polyclinics_ids_and_names: Vec<(i32, String)> =
        polyclinics.into_iter().map(|p| (p.id, p.name)).collect();

    // convert to Vec<i32>
    let employees_ids: Vec<i32> = employees.into_iter().map(|e| e.id).collect();

    let doctor_models = generate_doctor(
        (employees_ids.len() as f64 * 0.45) as i32,
        employees_ids,
        polyclinics_ids_and_names,
    );

    doctors::Entity::insert_many(doctor_models)
        .exec(txn)
        .await?;
    info!("✅ Doctors seeded successfully.");
    Ok(())
}
