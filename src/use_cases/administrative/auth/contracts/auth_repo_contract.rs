use async_trait::async_trait;
use entity::users;
use sea_orm::DatabaseConnection;

use crate::error_handling::app_error::AppError;

#[async_trait]
pub trait AuthRepoContract {
    async fn find_user_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<users::Model, AppError>;
    async fn update_last_login(db: &DatabaseConnection, username: &str) -> Result<(), AppError>;
}
