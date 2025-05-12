use anyhow::Error;
use async_trait::async_trait;
use chrono::Utc;
use entity::{
    patients::{self, ActiveModel},
    patients_visit_intent, queue_ticket,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseTransaction, EntityTrait,
    PaginatorTrait, QueryFilter,
};

use crate::dtos::triage::create_triage_request::{CreateTriageRequest, VisitType};

pub use super::triage_repo_contract::TriageTraitRepo;

pub struct TriageRepo;

#[async_trait]
impl TriageTraitRepo for TriageRepo {
    async fn find_or_create_patient(
        txn: &DatabaseTransaction,
        payload: &CreateTriageRequest,
    ) -> Result<patients::Model, Error> {
        if let Some(existing) = patients::Entity::find()
            .filter(patients::Column::NationalId.eq(&payload.national_id))
            .one(txn)
            .await?
        {
            return Ok(existing);
        }

        let model = ActiveModel {
            name: Set(payload.name.clone()),
            date_of_birth: Set(payload.date_of_birth),
            national_id: Set(payload.national_id.clone()),
            bpjs_number: Set(payload.bpjs_number.clone()),
            gender: Set(payload.gender.to_string()),
            emergency_contact_name: Set(payload.emergency_contact_name.clone()),
            emergency_contact_phone: Set(payload.emergency_contact_phone.clone()),
            emergency_contact_relationship: Set(payload.emergency_contact_relationship.clone()),
            blood_type: Set(payload.blood_type.to_string()),
            known_allergies: Set(Some(payload.known_allergies.clone().unwrap_or_default())),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        Ok(model.insert(txn).await?)
    }

    async fn create_visit_intent(
        txn: &DatabaseTransaction,
        patient_id: i32,
        payload: &CreateTriageRequest,
    ) -> Result<entity::patients_visit_intent::Model, anyhow::Error> {
        let model = patients_visit_intent::ActiveModel {
            patient_id: Set(patient_id),
            visit_type: Set(payload.visit_type.to_string()),
            referral_document_url: Set(payload.referral_document_url.clone()),
            status: Set("WAITING".into()),
            referral_validated: Set("PENDING".into()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        Ok(model.insert(txn).await?)
    }

    async fn create_queue_ticket(
        txn: &DatabaseTransaction,
        intent_id: i32,
        visit_type: &VisitType,
    ) -> Result<entity::queue_ticket::Model, anyhow::Error> {
        let count = queue_ticket::Entity::find()
            .filter(queue_ticket::Column::QueueType.eq(visit_type.to_string()))
            .count(txn)
            .await?;

        let model = queue_ticket::ActiveModel {
            visit_intent_id: Set(intent_id),
            queue_number: Set(count as i32 + 1),
            queue_type: Set(visit_type.to_string()),
            status: Set("WAITING".into()),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        Ok(model.insert(txn).await?)
    }
}
