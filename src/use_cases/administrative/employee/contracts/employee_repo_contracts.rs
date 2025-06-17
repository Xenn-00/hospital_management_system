use async_trait::async_trait;
use entity::employees;
use sea_orm::DatabaseTransaction;

use crate::{
    dtos::administrative::employment::employment_request::RegisterEmployeeRequest,
    error_handling::app_error::AppError,
};

#[async_trait]
pub trait EmployeeRepoContract {
    async fn insert_employee(
        db: &DatabaseTransaction,
        payload: RegisterEmployeeRequest,
        supervisor_id: i32,
    ) -> Result<employees::Model, AppError>;
}
