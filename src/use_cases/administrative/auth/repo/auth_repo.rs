use async_trait::async_trait;
use chrono::Utc;
use entity::users;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter,
};

use crate::{
    error_handling::app_error::AppError,
    use_cases::administrative::auth::contracts::auth_repo_contract::AuthRepoContract,
};

pub struct AuthRepo;

#[async_trait]
impl AuthRepoContract for AuthRepo {
    async fn find_user_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<users::Model, AppError> {
        if let Some(existing) = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        Err(AppError::NotFound(format!(
            "Username '{}' is not found",
            username
        )))
    }
    async fn update_last_login(db: &DatabaseConnection, username: &str) -> Result<(), AppError> {
        let existing = Self::find_user_by_username(db, username).await?;
        let mut model: users::ActiveModel = existing.into();
        model.last_login = Set(Some(Utc::now().naive_utc()));
        users::Entity::update(model).exec(db).await?;

        Ok(())
    }

    async fn register_new_user(
        txn: &DatabaseTransaction,
        payload: users::ActiveModel,
    ) -> Result<users::Model, AppError> {
        Ok(users::Entity::insert(payload)
            .exec_with_returning(txn)
            .await?)
    }
}
