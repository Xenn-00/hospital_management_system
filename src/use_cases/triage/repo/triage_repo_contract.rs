use async_trait::async_trait;
use entity::{patients, patients_visit_intent, queue_ticket, referral_documents};
use sea_orm::{DatabaseConnection, DatabaseTransaction};

use crate::{
    dtos::triage::{
        create_triage_request::{CreateTriageRequest, VisitType},
        response::TriageQueueItem,
    },
    error_handling::app_error::AppError,
};

#[async_trait]
pub trait TriageTraitRepo {
    async fn find_or_create_patient(
        txn: &DatabaseTransaction,
        payload: &CreateTriageRequest,
    ) -> Result<patients::Model, AppError>;
    async fn create_visit_intent(
        txn: &DatabaseTransaction,
        patient_id: i32,
        payload: &CreateTriageRequest,
    ) -> Result<patients_visit_intent::Model, AppError>;
    async fn create_queue_ticket(
        txn: &DatabaseTransaction,
        intent_id: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError>;
    async fn get_queue(
        db: &DatabaseConnection,
        visit_type: &VisitType,
    ) -> Result<Vec<TriageQueueItem>, AppError>;
    async fn get_status_by_queue_number(
        db: &DatabaseConnection,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError>;
    async fn update_visit_intent_status(
        txn: &DatabaseTransaction,
        visit_intent_id: i32,
        status: &str,
    ) -> Result<(), AppError>;
    async fn call_patient(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError>;
    async fn triage_queue_complete(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: VisitType,
    ) -> Result<queue_ticket::Model, AppError>;
    async fn cancel_patient_queue(
        txn: &DatabaseTransaction,
        queue_number: i32,
        visit_type: &VisitType,
    ) -> Result<queue_ticket::Model, AppError>;
    async fn upload_referral_docs(
        txn: &DatabaseTransaction,
        filename: String,
        visit_id: i32,
        patient_id: i32,
        file_bytes: &Vec<u8>,
        url: String,
    ) -> Result<referral_documents::Model, AppError>;
}
