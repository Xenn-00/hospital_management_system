use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::{
    dtos::triage::response::{
        TriagePatientCalled, TriagePatientCancel, TriageQueueComplete, TriageQueueItem,
        TriageQueueResponse, TriageQueueStatus,
    },
    error_handling::app_error::AppError,
    format_created_at, format_option_dt, parse_visit_type,
    use_cases::triage::repo::triage_repo::{self, TriageRepo},
    utils::helpers::{get_cache_data, set_cache_data},
};
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
        db: &DatabaseConnection,
        payload: CreateTriageRequest,
    ) -> Result<CreateTriageResponse, AppError> {
        let txn = db.begin().await?;

        if matches!(payload.visit_type, VisitType::BPJS) {
            let ok = payload
                .referral_document_url
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            if !ok {
                AppError::BadRequest(format!("Referral document ist required for BPJS patient"));
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
            payload.visit_type,
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

    async fn get_triage_queue(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
    ) -> Result<TriageQueueResponse, AppError> {
        let normalize_type: Result<VisitType, AppError> = parse_visit_type!(visit_type);

        let cache_key = format!("triage:queue:{}", normalize_type.as_ref().unwrap());

        if let Some(cached) = get_cache_data::<Vec<TriageQueueItem>>(&redis, &cache_key).await? {
            let result = TriageQueueResponse {
                visit_type: normalize_type.unwrap().to_string(),
                data: cached,
            };
            return Ok(result);
        }

        let response = <TriageRepo as triage_repo::TriageTraitRepo>::get_queue(
            &db,
            normalize_type.as_ref().unwrap(),
        )
        .await?;

        let result = TriageQueueResponse {
            visit_type: normalize_type.unwrap().to_string(),
            data: response.clone(),
        };

        set_cache_data(&redis, &cache_key, &result, 300).await?;

        Ok(TriageQueueResponse {
            visit_type,
            data: response,
        })
    }

    async fn get_triage_queue_status_by_id(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriageQueueStatus, AppError> {
        let normalize_type: Result<VisitType, AppError> = parse_visit_type!(visit_type);
        let cache_key = format!(
            "triage:queue:{}:id:{}",
            normalize_type.as_ref().unwrap(),
            &queue_number
        );

        if let Some(cached) = get_cache_data::<TriageQueueStatus>(&redis, &cache_key).await? {
            return Ok(cached);
        }

        let response = <TriageRepo as triage_repo::TriageTraitRepo>::get_status_by_queue_number(
            &db,
            queue_number,
            normalize_type.unwrap(),
        )
        .await?;

        let formatted = format_created_at!(response.created_at);

        let result = TriageQueueStatus {
            queue_number: response.queue_number,
            queue_type: response.queue_type,
            status: response.status,
            created_at: formatted,
        };

        set_cache_data(&redis, &cache_key, &result, 300).await?;

        Ok(result)
    }

    async fn call_patient(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriagePatientCalled, AppError> {
        let normalize_type: Result<VisitType, AppError> = parse_visit_type!(visit_type);
        let cache_key = format!(
            "call:queue:{}:id:{}",
            normalize_type.as_ref().unwrap(),
            &queue_number
        );

        if let Some(cached) = get_cache_data::<TriagePatientCalled>(&redis, &cache_key).await? {
            return Ok(cached);
        }

        let txn = db.begin().await?;

        let response = <TriageRepo as triage_repo::TriageTraitRepo>::call_patient(
            &txn,
            queue_number,
            normalize_type.unwrap(),
        )
        .await?;

        txn.commit().await.unwrap();

        let formatted = format_option_dt!(response.called_at);

        let result = TriagePatientCalled {
            queue_number: response.queue_number,
            queue_type: response.queue_type,
            called_at: formatted,
        };

        set_cache_data(&redis, &cache_key, &result, 300).await?;

        Ok(result)
    }

    async fn triage_complete(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriageQueueComplete, AppError> {
        let normalize_type: Result<VisitType, AppError> = parse_visit_type!(visit_type);
        let cache_key = format!(
            "complete:queue:{}:id:{}",
            normalize_type.as_ref().unwrap(),
            &queue_number
        );

        if let Some(cached) = get_cache_data::<TriageQueueComplete>(&redis, &cache_key).await? {
            return Ok(cached);
        }

        let txn = db.begin().await?;

        let response = <TriageRepo as triage_repo::TriageTraitRepo>::triage_queue_complete(
            &txn,
            queue_number,
            normalize_type.unwrap(),
        )
        .await?;

        txn.commit().await.unwrap();

        let formatted_called_at = format_option_dt!(response.called_at);
        let formatted_done_at = format_option_dt!(response.done_at);

        let result = TriageQueueComplete {
            queue_number: response.queue_number,
            queue_type: response.queue_type,
            status: response.status,
            called_at: formatted_called_at,
            done_at: formatted_done_at,
        };

        set_cache_data(&redis, &cache_key, &result, 300).await?;

        Ok(result)
    }

    async fn cancel_patient_queue(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriagePatientCancel, AppError> {
        let normalize_type: Result<VisitType, AppError> = parse_visit_type!(visit_type);
        let cache_key = format!(
            "cancel:queue:{}:id:{}",
            normalize_type.as_ref().unwrap(),
            &queue_number
        );
        if let Some(cached) = get_cache_data::<TriagePatientCancel>(&redis, &cache_key).await? {
            return Ok(cached);
        };

        let ticket = <TriageRepo as triage_repo::TriageTraitRepo>::get_status_by_queue_number(
            &db,
            queue_number.clone(),
            normalize_type.clone().unwrap(),
        )
        .await?;

        let txn = db.begin().await?;
        let response = <TriageRepo as triage_repo::TriageTraitRepo>::cancel_patient_queue(
            &txn,
            queue_number.clone(),
            &normalize_type.unwrap(),
        )
        .await?;

        txn.commit().await?;

        let result = TriagePatientCancel {
            queue_type: response.queue_type,
            queue_number: response.queue_number,
            previous_status: ticket.status,
            new_status: response.status,
        };

        set_cache_data(&redis, &cache_key, &result, 300).await?;

        Ok(result)
    }
}
