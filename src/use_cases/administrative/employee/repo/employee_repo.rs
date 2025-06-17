use async_trait::async_trait;
use chrono::NaiveDate;
use entity::employees;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
};

use crate::{
    dtos::administrative::employment::employment_request::RegisterEmployeeRequest,
    error_handling::app_error::AppError,
    use_cases::administrative::employee::contracts::employee_repo_contracts::EmployeeRepoContract,
};

pub struct EmployeeRepo;

#[async_trait]
impl EmployeeRepoContract for EmployeeRepo {
    async fn insert_employee(
        db: &DatabaseTransaction,
        payload: RegisterEmployeeRequest,
        supervisor_id: i32,
    ) -> Result<employees::Model, AppError> {
        // lets count nip
        if let Some(_) = employees::Entity::find()
            .filter(employees::Column::Nip.eq(payload.nip.as_deref()))
            .one(db)
            .await?
        {
            return Err(AppError::BadRequest(format!(
                "This NIP has been registered."
            )));
        }

        let birth_date = NaiveDate::parse_from_str(&payload.birth_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest(format!("Invalid birth date format")))?;

        let hire_date = NaiveDate::parse_from_str(&payload.hire_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest(format!("Invalid birth date format")))
            .expect("Invalid birth date format");

        let gender = match payload.gender.to_lowercase().as_str() {
            "male" => 1,
            "female" => 2,
            _ => return Err(AppError::BadRequest(format!("Gender is invalid"))),
        };

        let model = employees::ActiveModel {
            nip: Set(payload.nip),
            full_name: Set(payload.full_name),
            birth_date: Set(birth_date),
            hire_date: Set(hire_date),
            created_by: Set(Some(supervisor_id)),
            address: Set(payload.address),
            email: Set(payload.email),
            phone: Set(payload.phone),
            employment_status: Set(payload.employement_status),
            department_code: Set(payload.department_code),
            gender: Set(gender),
            ..Default::default()
        };

        Ok(model.insert(db).await?)
    }
}
