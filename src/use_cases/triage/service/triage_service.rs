use anyhow::bail;
use async_trait::async_trait;
use sea_orm::TransactionTrait;

use crate::use_cases::triage::repo::triage_repo::{self, TriageRepo};
pub use crate::{
    dtos::triage::{
        create_triage_request::{CreateTriageRequest, VisitType},
        response::CreateTriageResponse,
    },
    use_cases::triage::contracts::triage_contract::TriageContracts,
};

pub struct TriageService;

#[async_trait]
impl TriageContracts for TriageService {
    async fn perform_triage(
        db: &sea_orm::DatabaseConnection,
        payload: CreateTriageRequest,
    ) -> Result<CreateTriageResponse, anyhow::Error> {
        let txn = db.begin().await?;

        if matches!(payload.visit_type, VisitType::BPJS) {
            let ok = payload
                .referral_document_url
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            if !ok {
                bail!("Referral document is required for BPJS");
            }
        }

        let patient =
            <TriageRepo as triage_repo::TriageTraitRepo>::find_or_create_patient(&txn, &payload)
                .await?;

        let visit_intent = <TriageRepo as triage_repo::TriageTraitRepo>::create_visit_intent(
            &txn, patient.id, &payload,
        )
        .await?;

        let queue_ticket = <TriageRepo as triage_repo::TriageTraitRepo>::create_queue_ticket(
            &txn,
            visit_intent.id,
            &payload.visit_type,
        )
        .await?;

        txn.commit().await?;

        Ok(CreateTriageResponse {
            patient_id: patient.id,
            visit_intent_id: visit_intent.id,
            queue_number: queue_ticket.queue_number,
            queue_type: queue_ticket.queue_type,
            status: queue_ticket.status,
        })
    }
}
