use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}
