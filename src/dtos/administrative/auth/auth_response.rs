use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
pub struct AdminCreateEmployeeAccountResponse {
    pub employee_id: i32,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub message: String,
    pub employee_id: i32,
    pub temporary_setup_token: String,
}

#[derive(Debug, Serialize)]
pub struct EmployeeRegisterUserResponse {
    pub message: String,
    pub status: String,
    pub redirect_url: String,
}
