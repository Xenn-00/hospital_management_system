use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::utils::validation::employee_request_validation::{validate_password, validate_username};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateEmployeeAccountRequest {
    pub employee_id: i32,
    #[validate(length(min = 4, message = "Role is required"))]
    pub role: String,
    #[validate(length(min = 5, max = 5, message = "Invalid department code"))]
    pub department_code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    pub employee_id: i32,
    #[validate(length(min = 6, message = "Invalid otp code"))]
    pub otp_number: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeeRegisterUserRequest {
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupUserQuery {
    pub setup_token: String,
    pub employee_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct ResendOTPQuery {
    pub employee_id: i32,
}

#[derive(Debug, Serialize)]
pub struct SendOTPPayloadPubSub {
    pub to: String,
    pub otp: String,
}
