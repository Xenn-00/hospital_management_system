use serde::Deserialize;
use validator::Validate;

use crate::utils::validation::employee_request_validation::{
    validate_employee_id, validate_password,
};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(custom(function = "validate_employee_id"))]
    pub employee_id: String,
    #[validate(length(min = 4, message = "Username is too short"))]
    pub username: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
    #[validate(length(min = 4, message = "Role is required"))]
    pub role: String,
    #[validate(length(min = 5, max = 5, message = "Invalid department code"))]
    pub department_code: String,
}
