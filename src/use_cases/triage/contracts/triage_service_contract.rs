use async_trait::async_trait;
use aws_sdk_s3::Client;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::triage::{
        create_triage_request::CreateTriageRequest,
        referral_upload_metadata::ReferralUploadMetadata,
        response::{
            CreateTriageResponse, ReferralUploadResponse, TriagePatientCalled, TriagePatientCancel,
            TriageQueueComplete, TriageQueueResponse, TriageQueueStatus,
        },
    },
    error_handling::app_error::AppError,
};

#[async_trait]
pub trait TriageServiceContracts {
    async fn perform_triage(
        db: &DatabaseConnection,
        payload: CreateTriageRequest,
    ) -> Result<CreateTriageResponse, AppError>;
    async fn get_triage_queue(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
    ) -> Result<TriageQueueResponse, AppError>;
    async fn get_triage_queue_status_by_id(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriageQueueStatus, AppError>;
    async fn call_patient(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriagePatientCalled, AppError>;
    async fn triage_complete(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriageQueueComplete, AppError>;
    async fn cancel_patient_queue(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        visit_type: String,
        queue_number: i32,
    ) -> Result<TriagePatientCancel, AppError>;
    async fn handle_referral_upload(
        db: &DatabaseConnection,
        s3: &Client,
        visit_id: i32,
        patient_id: i32,
        metadata: ReferralUploadMetadata,
    ) -> Result<ReferralUploadResponse, AppError>;
}
