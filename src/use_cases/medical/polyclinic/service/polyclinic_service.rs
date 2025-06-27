use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use entity::polyclinic;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use tokio::time::Instant;

use crate::{
    dtos::medical::medical_response::{
        PagedPolyclinicSchedulesResponse, PolyclinicSchedulesResponse,
    },
    error_handling::app_error::AppError,
    infra::api::{PaginationMeta, PaginationQuery},
    use_cases::medical::polyclinic::{
        contracts::{
            polyclinic_repo_contract::PolyclinicRepoContract,
            polyclinic_service_contract::PolyclinicServiceContract,
        },
        repo::polyclinic_repo::PolyclinicRepo,
    },
    utils::helpers::{get_cache_data, set_cache_data},
};

pub struct PolyclinicService;

#[async_trait]
impl PolyclinicServiceContract for PolyclinicService {
    async fn polyclinic_schedules(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        poly_code: &String,
    ) -> Result<PolyclinicSchedulesResponse, AppError> {
        // 1. check cache
        let now = Instant::now();
        let cache_key = format!("polyclinic:{}", poly_code);
        if let Some(cached) =
            get_cache_data::<PolyclinicSchedulesResponse>(redis, &cache_key).await?
        {
            return Ok(cached);
        };

        tracing::info!("redis latency: {:?}", now.elapsed());

        tracing::info!("[Redis] {} is unreachable", &cache_key);
        tracing::info!("[DB] Continue in DB");

        // // 2. check polyclinic_id in db
        let exists = match polyclinic::Entity::find()
            .filter(polyclinic::Column::Code.eq(poly_code))
            .count(db)
            .await
        {
            Ok(exist) => exist,
            Err(e) => {
                tracing::error!(
                    "[DB] Can't find polyclinic with id {}, error: {}",
                    poly_code,
                    e
                );
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when find polyclinic {}",
                    poly_code
                )));
            }
        };

        if !(exists > 0) {
            return Err(AppError::NotFound(format!("Polyclinic not found")));
        }

        // // 3. find schedule + responsible doctor
        let response =
            <PolyclinicRepo as PolyclinicRepoContract>::find_poly_schedule(db, poly_code).await?;

        match set_cache_data(redis, &cache_key, &response, 300).await {
            Ok(()) => (),
            Err(e) => {
                tracing::error!("[Redis] Failed to set cache, error: {}", e);
                return Err(AppError::Internal(format!(
                    "Unexpected error occur when set cache"
                )));
            }
        }
        // // 4. return
        Ok(response)
    }

    async fn get_all_schedules(
        db: &DatabaseConnection,
        redis: &Pool<RedisConnectionManager>,
        paging: PaginationQuery,
    ) -> Result<PagedPolyclinicSchedulesResponse, AppError> {
        let page = paging.page.unwrap_or(1);
        let per_page = paging.per_page.unwrap_or(10).max(1);
        let offset = (page - 1) * per_page;
        // 1. check cache
        let cache_key = format!("polyclinic:schedules:page:{}:per_page:{}", page, per_page);

        if let Some(cached) =
            get_cache_data::<PagedPolyclinicSchedulesResponse>(redis, &cache_key).await?
        {
            return Ok(cached);
        };

        tracing::info!("[Redis] {} is unreachable", &cache_key);
        tracing::info!("[DB] Continue in DB");
        // 2. find all polyclinic schedules

        let (total, data) =
            <PolyclinicRepo as PolyclinicRepoContract>::get_all_schedules(db, offset, per_page)
                .await?;

        // 3. create response

        let total_pages = (total + per_page - 1) / per_page;

        let response = PagedPolyclinicSchedulesResponse {
            data,
            meta: PaginationMeta {
                total,
                page,
                per_page,
                total_pages,
            },
        };

        set_cache_data(redis, &cache_key, &response, 300).await?;

        //  4. return response
        Ok(response)
    }
}
