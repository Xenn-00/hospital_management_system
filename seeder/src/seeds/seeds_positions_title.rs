use std::collections::HashMap;

use entity::{departments, position_titles};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::generate_position_title;

pub async fn seeds_positions_title(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding positions title");
    let count = position_titles::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️ Employees position already seeded. Skipping...");
        return Ok(());
    }

    let all_dept = departments::Entity::find().all(txn).await?;

    let dept_map: HashMap<String, String> = all_dept
        .into_iter()
        .map(|dept| (dept.name.clone(), dept.code.clone()))
        .collect();

    let positions: Vec<(&'static str, &'static str, &'static str)> = vec![
        (
            "Doctor",
            "Handles patient diagnosis and treatment",
            "Clinical",
        ),
        ("Nurse", "Assists doctors and cares for patients", "Nursing"),
        (
            "Admin Staff",
            "Handles admin tasks and paperwork",
            "Administrative",
        ),
        (
            "Finance Officer",
            "Manages hospital budgeting and expenses",
            "Finance",
        ),
        (
            "Procurement Officer",
            "Manages medical supply acquisitions",
            "Procurement",
        ),
        ("IT Support", "Maintains hospital IT infrastructure", "IT"),
        (
            "Lab Technician",
            "Performs lab tests and analyses",
            "Laboratory",
        ),
        (
            "HR Specialist",
            "Handles hiring, payroll, and employee relations",
            "Human Resource",
        ),
        (
            "Support Staff",
            "General hospital maintenance and services",
            "Support",
        ),
        (
            "Emergency Physician",
            "Works in ER handling critical cases",
            "Emergency",
        ),
    ];

    let models = generate_position_title(positions, dept_map);

    position_titles::Entity::insert_many(models)
        .exec(txn)
        .await?;
    info!("✅ Positions title seeded successfully.");
    Ok(())
}
