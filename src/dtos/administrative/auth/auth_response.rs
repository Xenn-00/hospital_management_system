use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterUserResponse {
    pub username: String,
    pub role: String,
    pub is_active: bool,
}
