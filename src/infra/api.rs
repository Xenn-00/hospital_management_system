use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub message: String,
    pub data: Option<T>,
    pub request_id: String,
    pub errors: Option<Vec<ApiFieldError>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ApiFieldError {
    pub field: String,
    pub message: String,
}
