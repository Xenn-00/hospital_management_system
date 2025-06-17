use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::administrative::employment::{
        employment_request::RegisterEmployeeRequest, employment_response::RegisterEmployeeResponse,
    },
    error_handling::app_error::AppError,
    utils::jwt::Claims,
};

#[async_trait]
pub trait EmployeeServiceContract {
    async fn register_employee(
        db: &DatabaseConnection,
        payload: RegisterEmployeeRequest,
        claim: Claims,
    ) -> Result<RegisterEmployeeResponse, AppError>;
}
