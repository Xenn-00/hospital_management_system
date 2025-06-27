use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use entity::{doctor_schedules, doctors, polyclinic};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    RelationTrait,
};

use crate::{
    dtos::medical::medical_response::{PolySchedules, PolyclinicSchedulesResponse},
    error_handling::app_error::AppError,
    use_cases::medical::polyclinic::contracts::polyclinic_repo_contract::PolyclinicRepoContract,
};

pub struct PolyclinicRepo;

#[async_trait]
impl PolyclinicRepoContract for PolyclinicRepo {
    async fn find_poly_schedule(
        db: &DatabaseConnection,
        poly_code: &String,
    ) -> Result<PolyclinicSchedulesResponse, AppError> {
        let rows: Vec<(
            polyclinic::Model,
            Option<doctors::Model>,
            Option<doctor_schedules::Model>,
        )> = match polyclinic::Entity::find()
            .filter(polyclinic::Column::Code.eq(poly_code))
            .join(
                sea_orm::JoinType::InnerJoin,
                polyclinic::Relation::Doctors.def(),
            )
            .join(
                sea_orm::JoinType::InnerJoin,
                doctors::Relation::DoctorSchedules.def(),
            )
            .select_also(doctors::Entity)
            .select_also(doctor_schedules::Entity)
            .all(db)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("[DB] Failed to fetch polyclinic schedule: {}", e);
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when trying to fetch polyclinic schedule"
                )));
            }
        };

        if rows.is_empty() {
            return Err(AppError::NotFound(format!(
                "No Schedule found for this polyclinic {}",
                poly_code
            )));
        }

        let poly = rows.first().map(|(p, _, _)| p).ok_or_else(|| {
            AppError::NotFound(format!("No schedule found for polyclinic {}", poly_code))
        })?;

        let mut doctor_names = HashSet::new();
        let mut schedules = Vec::new();
        let mut room_code = None;

        for (_, doctor_opt, schedule_opt) in &rows {
            if let Some(doctor) = doctor_opt {
                doctor_names.insert(doctor.name.clone());
            }
            if let Some(sched) = schedule_opt {
                if room_code.is_none() {
                    room_code = Some(sched.room_code.clone());
                }
                schedules.push(PolySchedules {
                    day_of_week: sched.day_of_week.clone(),
                    start_time: sched.start_time.clone(),
                    end_time: sched.end_time.clone(),
                });
            }
        }

        Ok(PolyclinicSchedulesResponse {
            polyclinic_code: poly_code.to_string(),
            polyclinic_name: poly.name.clone(),
            room_code: room_code.ok_or_else(|| {
                AppError::Internal("Room code missing in schedule data".to_string())
            })?,
            doctor_responsible: doctor_names.into_iter().collect(),
            schedules,
        })
    }

    async fn get_all_schedules(
        db: &DatabaseConnection,
        offset: i32,
        limit: i32,
    ) -> Result<(i32, Vec<PolyclinicSchedulesResponse>), AppError> {
        let total = match polyclinic::Entity::find()
            .select_only()
            .column(polyclinic::Column::Code)
            .distinct()
            .count(db)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                tracing::error!("[DB] Failed to count polyclinic schedules: {}", e);
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when trying to count polyclinic schedules"
                )));
            }
        } as i32;

        let poly_ids_rows: Vec<i32> = match polyclinic::Entity::find()
            .offset(offset as u64)
            .limit(limit as u64)
            .all(db)
            .await
        {
            Ok(rows) => rows.iter().map(|p| p.id).collect(),
            Err(e) => {
                tracing::error!("[DB] Failed to fetch polyclinic rows: {}", e);
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when trying to fetch polyclinic rows"
                )));
            }
        };

        let rows: Vec<(
            polyclinic::Model,
            Option<doctors::Model>,
            Option<doctor_schedules::Model>,
        )> = match polyclinic::Entity::find()
            .filter(polyclinic::Column::Id.is_in(poly_ids_rows.clone()))
            .join(
                sea_orm::JoinType::InnerJoin,
                polyclinic::Relation::Doctors.def(),
            )
            .join(
                sea_orm::JoinType::InnerJoin,
                doctors::Relation::DoctorSchedules.def(),
            )
            .select_also(doctors::Entity)
            .select_also(doctor_schedules::Entity)
            .all(db)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("[DB] Failed to fetch all polyclinic schedules: {}", e);
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when trying to fetch all polyclinic schedules"
                )));
            }
        };

        if rows.is_empty() {
            return Ok((total, Vec::new()));
        }

        tracing::info!("[DB] Retrieved {} polyclinic schedules", rows.len());

        let mut map = HashMap::new();

        for (poly, doctor_opt, schedule_opt) in rows {
            let entry =
                map.entry(poly.code.clone())
                    .or_insert_with(|| PolyclinicSchedulesResponse {
                        polyclinic_code: poly.code.clone(),
                        polyclinic_name: poly.name.clone(),
                        room_code: poly.room_code.clone(),
                        doctor_responsible: Vec::new(),
                        schedules: Vec::new(),
                    });

            if let Some(doctor) = doctor_opt {
                if !entry.doctor_responsible.contains(&doctor.name) {
                    entry.doctor_responsible.push(doctor.name.clone());
                }
            }

            if let Some(sched) = schedule_opt {
                entry.schedules.push(PolySchedules {
                    day_of_week: sched.day_of_week.clone(),
                    start_time: sched.start_time.clone(),
                    end_time: sched.end_time.clone(),
                });
            }
        }

        let polyclinic_schedules: Vec<PolyclinicSchedulesResponse> = map.into_values().collect();

        tracing::info!(
            "[Serivce] Retrieved {} polyclinic schedules",
            polyclinic_schedules.len()
        );

        Ok((total, polyclinic_schedules))
    }
}
