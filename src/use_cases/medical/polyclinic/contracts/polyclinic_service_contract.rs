use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use crate::{
    dtos::medical::medical_response::{
        PagedPolyclinicSchedulesResponse, PolyclinicSchedulesResponse,
    },
    error_handling::app_error::AppError,
    infra::api::PaginationQuery,
};

#[async_trait]
pub trait PolyclinicServiceContract {
    async fn polyclinic_schedules(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        poly_code: &String,
    ) -> Result<PolyclinicSchedulesResponse, AppError>;
    async fn get_all_schedules(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        paging: PaginationQuery,
    ) -> Result<PagedPolyclinicSchedulesResponse, AppError>;
}
