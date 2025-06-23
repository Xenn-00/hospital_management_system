use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use entity::{
    patients::{self, ActiveModel},
    patients_visit_intent, queue_sequence, queue_ticket, referral_documents,
};

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::{
    dtos::triage::{
        create_triage_request::{CreateTriageRequest, VisitType},
        response::TriageQueueItem,
    },
    error_handling::app_error::AppError,
    format_created_at,
    use_cases::triage::contracts::triage_repo_contract::TriageTraitRepo,
};

pub struct TriageRepo;

enum QueueStatus {
    WAITING,
    CALLED,
    DONE,
    CANCELLED,
}

enum ReferralStatus {
    WAITING,
}

impl ToString for ReferralStatus {
    fn to_string(&self) -> String {
        match self {
            ReferralStatus::WAITING => "WAITING",
        }
        .to_string()
    }
}

impl ToString for QueueStatus {
    fn to_string(&self) -> String {
        match self {
            QueueStatus::WAITING => "WAITING",
            QueueStatus::CALLED => "CALLED",
            QueueStatus::DONE => "DONE",
            QueueStatus::CANCELLED => "CANCELED",
        }
        .to_string()
    }
}

#[async_trait]
impl TriageTraitRepo for TriageRepo {
    async fn find_or_create_patient(
        txn: &DatabaseTransaction,
        payload: &CreateTriageRequest,
    ) -> Result<patients::Model, AppError> {
        if let Some(existing) = patients::Entity::find()
            .filter(patients::Column::NationalId.eq(&payload.national_id))
            .one(txn)
            .await?
        {
            return Ok(existing);
        }

        let now = Utc::now().naive_utc();
        let model = ActiveModel {
            name: Set(payload.name.to_owned()),
            date_of_birth: Set(payload.date_of_birth),
            national_id: Set(payload.national_id.to_owned()),
            bpjs_number: Set(payload.bpjs_number.to_owned()),
            gender: Set(payload.gender.to_string()),
            emergency_contact_name: Set(payload.emergency_contact_name.to_owned()),
            emergency_contact_phone: Set(payload.emergency_contact_phone.to_owned()),
            emergency_contact_relationship: Set(payload.emergency_contact_relationship.to_owned()),
            blood_type: Set(payload.blood_type.to_string()),
            known_allergies: Set(Some(payload.known_allergies.to_owned().unwrap_or_default())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        Ok(model.insert(txn).await?)
    }

    async fn create_visit_intent(
        txn: &DatabaseTransaction,
        patient_id: i32,
        payload: &CreateTriageRequest,
    ) -> Result<entity::patients_visit_intent::Model, AppError> {
        let now = Utc::now().naive_utc();
        let model = patients_visit_intent::ActiveModel {
            patient_id: Set(patient_id),
            visit_type: Set(payload.visit_type.to_string()),
            status: Set(QueueStatus::WAITING.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        Ok(model.insert(txn).await?)
    }

    async fn get_next_queue_number(
        txn: &DatabaseTransaction,
        visit_type: &VisitType,
    ) -> Result<i32, AppError> {
        let visit = visit_type.to_string();

        if let Some(mut model) = queue_sequence::Entity::find()
            .filter(queue_sequence::Column::VisitType.eq(&visit))
            .lock_exclusive()
            .one(txn)
            .await?
        {
            model.last_number += 1;

            let active: queue_sequence::ActiveModel = model.into();
            let updated = active.update(txn).await?;

            Ok(updated.last_number)
        } else {
            let new_model = queue_sequence::ActiveModel {
                visit_type: Set(visit),
                last_number: Set(1),
                ..Default::default()
            }
            .insert(txn)
            .await?;

            Ok(new_model.last_number)
        }
    }

    async fn create_queue_ticket(
        txn: &DatabaseTransaction,
        intent_id: i32,
        visit_type: VisitType,
    ) -> Result<entity::queue_ticket::Model, AppError> {
        let tracking_number = Self::get_next_queue_number(txn, &visit_type).await?;

        let model = queue_ticket::ActiveModel {
            visit_intent_id: Set(intent_id),
            queue_number: Set(tracking_number),
            queue_type: Set(visit_type.to_string()),
            status: Set(QueueStatus::WAITING.to_string()),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        Ok(model.insert(txn).await?)
    }

    async fn get_queue(
        db: &DatabaseConnection,
        visit_type: &VisitType,
        offset: i32,
        limit: i32,
    ) -> Result<(i32, Vec<TriageQueueItem>), AppError> {
        let visit_string = visit_type.to_string();

        let total = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueType.eq(&visit_string))
            .filter(queue_ticket::Column::Status.eq(QueueStatus::WAITING.to_string()))
            .count(db)
            .await? as i32;

        let entries = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueType.eq(&visit_string))
            .filter(queue_ticket::Column::Status.eq(QueueStatus::WAITING.to_string()))
            .order_by_asc(queue_ticket::Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .find_also_related(patients_visit_intent::Entity)
            .all(db)
            .await?;

        let data = entries
            .into_iter()
            .filter_map(|(ticket, visit)| {
                visit.map(|v| TriageQueueItem {
                    queue_number: ticket.queue_number,
                    patient_id: v.patient_id,
                    visit_id: ticket.visit_intent_id,
                    status: ticket.status,
                    created_at: format_created_at!(ticket.created_at),
                })
            })
            .collect();

        Ok((total, data))
    }

    async fn get_status_by_queue_number(
        db: &DatabaseConnection,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError> {
        if let Some(existing) = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueNumber.eq(queue_number))
            .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string()))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        Err(AppError::NotFound(format!(
            "Queue {} is not found with number {}",
            visit_type, queue_number
        )))
    }

    async fn update_visit_intent_status(
        txn: &DatabaseTransaction,
        visit_intent_id: i32,
        status: String,
    ) -> Result<(), AppError> {
        let intent = patients_visit_intent::Entity::find_by_id(visit_intent_id)
            .one(txn)
            .await?
            .ok_or(AppError::NotFound("Visit intent not found".into()))?;

        let mut active = patients_visit_intent::ActiveModel::from(intent);
        active.status = Set(status.into());
        active.updated_at = Set(Utc::now().naive_utc());
        active.update(txn).await?;
        Ok(())
    }

    async fn call_patient(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError> {
        let ticket = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueNumber.eq(queue_number))
            .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string()))
            .lock_exclusive()
            .one(txn)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Queue {} with number {} not found",
                    visit_type, queue_number
                ))
            })?;

        match ticket.status.to_uppercase().as_str() {
            "WAITING" => {
                let mut active: queue_ticket::ActiveModel = ticket.into();
                active.status = Set(QueueStatus::CALLED.to_string());
                active.called_at = Set(Some(Utc::now().naive_utc()));
                let updated = active.update(txn).await?;
                Self::update_visit_intent_status(
                    txn,
                    updated.visit_intent_id,
                    QueueStatus::CALLED.to_string(),
                )
                .await?;
                Ok(updated)
            }
            "CALLED" => Err(AppError::BadRequest(format!(
                "Queue {} with number {} is already being called",
                visit_type, queue_number
            ))),
            "DONE" => Err(AppError::BadRequest(format!(
                "Queue {} with number {} has already been completed",
                visit_type, queue_number
            ))),
            s => Err(AppError::Internal(format!("Invalid queue status: {}", s))),
        }
    }

    async fn triage_queue_complete(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError> {
        if let Some(ticket) = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueNumber.eq(queue_number))
            .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string()))
            .one(txn)
            .await?
        {
            if ticket.status != "CALLED" {
                return Err(AppError::BadRequest(format!(
                    "Queue {} with number {} is not currently being called. status: {}",
                    visit_type, queue_number, ticket.status
                )));
            }
            let mut active: queue_ticket::ActiveModel = ticket.into();
            active.status = Set(QueueStatus::DONE.to_string());
            active.done_at = Set(Some(Utc::now().naive_utc()));
            let updated = active.update(txn).await?;
            Self::update_visit_intent_status(
                txn,
                updated.visit_intent_id,
                QueueStatus::DONE.to_string(),
            )
            .await?;
            return Ok(updated);
        }

        Err(AppError::NotFound(format!(
            "Queue {} with number {} is not found",
            visit_type, queue_number
        )))
    }

    async fn cancel_patient_queue(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: &VisitType,
    ) -> Result<queue_ticket::Model, AppError> {
        if let Some(ticket) = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueNumber.eq(queue_number))
            .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string()))
            .one(txn)
            .await?
        {
            if ticket.status == QueueStatus::CALLED.to_string()
                || ticket.status == QueueStatus::DONE.to_string()
            {
                return Err(AppError::BadRequest(format!(
                    "Queue {} with number {} is currently being called",
                    visit_type, queue_number
                )));
            }
            let mut active: queue_ticket::ActiveModel = ticket.into();
            active.status = Set(QueueStatus::CANCELLED.to_string());
            let updated = active.update(txn).await?;
            return Ok(updated);
        }

        Err(AppError::NotFound(format!(
            "Queue {} with number {} is not found",
            visit_type, queue_number
        )))
    }

    async fn upload_referral_docs(
        txn: &DatabaseTransaction,
        filename: String,
        visit_id: i32,
        patient_id: i32,
        file_bytes: &Vec<u8>,
        url: String,
    ) -> Result<referral_documents::Model, AppError> {
        let model = referral_documents::ActiveModel {
            file_name: Set(filename),
            visit_intent_id: Set(visit_id),
            patients_id: Set(patient_id),
            file_size: Set((file_bytes.len()) as i64),
            status: Set(ReferralStatus::WAITING.to_string()),
            referral_document_url: Set(url),
            ..Default::default()
        }
        .insert(txn)
        .await?;

        let intent = patients_visit_intent::Entity::find_by_id(visit_id)
            .one(txn)
            .await?
            .ok_or(AppError::NotFound("Visit intent not found".into()))?;
        let mut active = patients_visit_intent::ActiveModel::from(intent);
        active.referral_document_id = Set(Some(model.id));
        active.update(txn).await?;

        Ok(model)
    }
}
