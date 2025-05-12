use anyhow::Error;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::dtos::triage::{
    create_triage_request::CreateTriageRequest, response::CreateTriageResponse,
};

#[async_trait]
pub trait TriageContracts {
    async fn perform_triage(
        db: &DatabaseConnection,
        payload: CreateTriageRequest,
    ) -> Result<CreateTriageResponse, Error>;
}
