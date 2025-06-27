use std::collections::HashMap;

use chrono::NaiveTime;
use entity::{doctor_schedules, doctors, polyclinic};
use log::{info, warn};
use sea_orm::{DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait};

use crate::helpers::{generate_doctor_schedules, DoctorPoly};

pub async fn seeds_doctor_schedules(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding doctor schedules...");
    let count = entity::doctor_schedules::Entity::find().count(txn).await?;
    if count > 0 {
        info!("⚠️  Doctors already seeded. Skipping...");
        return Ok(());
    }

    let doctors = doctors::Entity::find().all(txn).await?;
    let polyclinics = polyclinic::Entity::find().all(txn).await?;

    let poly_map: HashMap<String, (i32, String)> = polyclinics
        .iter()
        .map(|p| (p.name.to_lowercase(), (p.id, p.room_code.clone())))
        .collect();

    let mut doctor_meta = Vec::new();

    for doctor in doctors {
        if let Some((poly_id, room_code)) = poly_map.get(&doctor.specialization.to_lowercase()) {
            doctor_meta.push(DoctorPoly {
                id: doctor.id,
                poly_id: *poly_id,
                room_code: room_code.clone(),
            });
        } else {
            warn!(
                "⚠️ Doctor {} specialization {} has no matching polyclinic",
                doctor.name, doctor.specialization
            );
        }
    }

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
