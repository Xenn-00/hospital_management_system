use anyhow::Error;
use async_trait::async_trait;
use entity::{patients, patients_visit_intent, queue_ticket};
use sea_orm::DatabaseTransaction;

use crate::dtos::triage::create_triage_request::{CreateTriageRequest, VisitType};

#[async_trait]
pub trait TriageTraitRepo {
    async fn find_or_create_patient(
        txn: &DatabaseTransaction,
        payload: &CreateTriageRequest,
    ) -> Result<patients::Model, Error>;
    async fn create_visit_intent(
        txn: &DatabaseTransaction,
        patient_id: i32,
        payload: &CreateTriageRequest,
    ) -> Result<patients_visit_intent::Model, Error>;
    async fn create_queue_ticket(
        txn: &DatabaseTransaction,
        intent_id: i32,
        visit_type: &VisitType,
    ) -> Result<queue_ticket::Model, Error>;
}
