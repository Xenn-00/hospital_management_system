use async_trait::async_trait;
use sea_orm::{DatabaseConnection, TransactionTrait};

use chrono::{DateTime, Local, NaiveDateTime, Utc};

use crate::{
    dtos::administrative::employment::{
        employment_request::RegisterEmployeeRequest, employment_response::RegisterEmployeeResponse,
    },
    error_handling::app_error::AppError,
    format_created_at,
    use_cases::administrative::employee::{
        contracts::{
            employee_repo_contracts::EmployeeRepoContract,
            employee_service_contract::EmployeeServiceContract,
        },
        repo::employee_repo::EmployeeRepo,
    },
    utils::jwt::Claims,
};

pub struct EmployeeService;

#[async_trait]
impl EmployeeServiceContract for EmployeeService {
    async fn register_employee(
        db: &DatabaseConnection,
        payload: RegisterEmployeeRequest,
        claim: Claims,
    ) -> Result<RegisterEmployeeResponse, AppError> {
        let txn = db.begin().await?;
        let supervisor_id = claim.sub;

        let register_employee =
            <EmployeeRepo as EmployeeRepoContract>::insert_employee(&txn, payload, supervisor_id)
                .await?;

        txn.commit().await?;

        let result = RegisterEmployeeResponse {
            id: register_employee.id,
            nip: register_employee.nip,
            full_name: register_employee.full_name,
            employment_status: register_employee.employment_status,
            department_code: register_employee.department_code,
            hire_date: register_employee.hire_date.into(),
            created_at: format_created_at!(register_employee.created_at),
            created_by: register_employee.created_by,
        };

        Ok(result)
    }
}
