use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::medical::medical_response::PolyclinicSchedulesResponse,
    error_handling::app_error::AppError,
};

#[async_trait]
pub trait PolyclinicRepoContract {
    async fn find_poly_schedule(
        db: &DatabaseConnection,
        poly_code: &String,
    ) -> Result<PolyclinicSchedulesResponse, AppError>;
    async fn get_all_schedules(
        db: &DatabaseConnection,
        offset: i32,
        limit: i32,
    ) -> Result<(i32, Vec<PolyclinicSchedulesResponse>), AppError>;
}
