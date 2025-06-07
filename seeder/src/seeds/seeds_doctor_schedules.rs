use chrono::NaiveTime;
use entity::{doctor_schedules, doctors};
use log::info;
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::{generate_doctor_schedules, Doctor};

pub async fn seeds_doctor_schedules(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding doctor schedules...");
    let count = entity::doctor_schedules::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Doctors already seeded. Skipping...");
        return Ok(());
    }

    let doctors = doctors::Entity::find().all(txn).await?;
    let doctor_meta: Vec<Doctor> = doctors
        .into_iter()
        .map(|doc| Doctor {
            id: doc.id,
            poly_id: doc.polyclinic_id,
            room_code: doc.room_code,
        })
        .collect();

    let days = vec![
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    let shifts = vec![
        (
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        ),
        (
            NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        ),
    ];

    let models = generate_doctor_schedules(doctor_meta, days, shifts);

    doctor_schedules::Entity::insert_many(models)
        .exec(txn)
        .await?;
    Ok(())
}
