use entity::{employee_position, employees, position_titles};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_employee_position;

pub async fn seeds_employees_position(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding employees position");
    let count = employee_position::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️ Employees position already seeded. Skipping...");
        return Ok(());
    }
    let employees = employees::Entity::find().all(txn).await?;
    let positions = position_titles::Entity::find().all(txn).await?;

    let employees_id_and_dept: Vec<(i32, String)> = employees
        .into_iter()
        .map(|emp| (emp.id, emp.department_code))
        .collect();
    let position_title_id_and_dept: Vec<(i32, String)> = positions
        .into_iter()
        .map(|pos| (pos.id, pos.department_code))
        .collect();

    let models = generate_employee_position(employees_id_and_dept, position_title_id_and_dept);

    employee_position::Entity::insert_many(models)
        .exec(txn)
        .await?;
    info!("✅ Employees position seeded successfully.");
    Ok(())
}
