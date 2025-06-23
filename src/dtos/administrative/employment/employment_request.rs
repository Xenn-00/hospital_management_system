use crate::utils::validation::employee_request_validation::{
    validate_birth_date, validate_department_code, validate_gender, validate_hire_date,
    validate_nip, validate_phone_number, validate_status_employee,
};
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterEmployeeRequest {
    #[validate(length(min = 1, message = "full name can't be blank"))]
    pub full_name: String,
    #[validate(custom(function = "validate_gender"))]
    pub gender: String,
    #[validate(length(min = 18, max = 18, message = "invalid nip given"))]
    pub nip: Option<String>,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_phone_number"))]
    pub phone: String,
    #[validate(custom(function = "validate_birth_date"))]
    pub birth_date: String, // format YYYY-MM-DD
    #[validate(custom(function = "validate_hire_date"))]
    pub hire_date: String, // format YYYY-MM-DD
    #[validate(length(min = 1, message = "address can't be blank"))]
    pub address: String,
    #[validate(custom(function = "validate_status_employee"))]
    pub employement_status: String,
    #[validate(custom(function = "validate_department_code"))]
    pub department_code: String,
}

impl RegisterEmployeeRequest {
    pub fn validate_cross_fields(&self) -> Result<(), ValidationError> {
        validate_nip(
            self.nip.as_ref().unwrap_or(&"".to_string()),
            &self.birth_date,
            &self.gender,
        )
    }
}
