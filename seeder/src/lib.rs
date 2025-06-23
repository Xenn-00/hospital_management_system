use log::info;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};

pub mod helpers;
pub mod seeds;

#[cfg(test)]
pub mod tests;

pub async fn seed_all(db: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    info!("🚀 Starting database seeding...");
    seeds::seeds_rooms::seeds_rooms(&txn).await?;
    seeds::seeds_roles::seeds_roles(&txn).await?;
    seeds::seeds_departments::seeds_departments(&txn).await?;
    seeds::seeds_positions_title::seeds_positions_title(&txn).await?;
    seeds::seeds_employees::seeds_employees(&txn).await?;
    seeds::seeds_polyclinic::seeds_polyclinic(&txn).await?;
    seeds::seeds_doctors::seeds_doctors(&txn).await?;
    seeds::seeds_employees_position::seeds_employees_position(&txn).await?;
    seeds::seeds_doctor_schedules::seeds_doctor_schedules(&txn).await?;
    seeds::seeds_nurses::seeds_nurses(&txn).await?;
    seeds::seeds_nurse_polyclinic_assignments::seeds_nurse_polyclinic_assignments(&txn).await?;
    seeds::seeds_department_roles::seeds_department_roles(&txn).await?;
    seeds::seeds_users::seeds_users(&txn).await?;
    txn.commit().await?;
    info!("✅ All seeds have been successfully applied.");
    Ok(())
}
