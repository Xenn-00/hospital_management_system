use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RegisterEmployeeResponse {
    pub id: i32,
    pub full_name: String,
    pub nip: Option<String>,
    pub hire_date: NaiveDateTime,
    pub employment_status: String,
    pub department_code: String,
    pub created_at: String,
    pub created_by: Option<i32>,
}
