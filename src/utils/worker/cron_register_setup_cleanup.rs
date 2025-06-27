use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::{Duration, Utc};
use entity::sea_orm_active_enums::AccountStatus;
use entity::users;
use redis::AsyncCommands;
use sea_orm::prelude::Expr;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityOrSelect, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect,
};
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Debug, FromQueryResult)]
struct PartialUser {
    id: i32,
    employee_id: i32,
}

pub async fn cron_register_setup_cleanup(
    db: DatabaseConnection,
    redis: Pool<RedisConnectionManager>,
) {
    let sched = match JobScheduler::new().await {
        Ok(sch) => sch,
        Err(e) => {
            tracing::error!("[Worker] Failed to initiate JobScheduler for register setup: {e}");
            return;
        }
    };

    match sched
        .add(
            Job::new_async("0 */5 * * * *", move |_, _| {
                let db = db.clone();
                let redis = redis.clone();
                Box::pin(async move {
                    tracing::info!("[Worker] Running cleanup at: {}", Utc::now());

                    let mut redis_conn = match redis.get().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            tracing::error!("[Worker] Failed to get Redis connection: {}", e);
                            return;
                        }
                    };

                    let Ok(bad_users) = users::Entity::find()
                        .filter(users::Column::AccountStatus.eq(AccountStatus::AwaitingSetup))
                        .filter(users::Column::Username.is_null())
                        .filter(users::Column::Password.is_null())
                        .filter(users::Column::CreatedAt.lt(Utc::now() - Duration::minutes(5)))
                        .select()
                        .column(users::Column::Id)
                        .column(users::Column::EmployeeId)
                        .into_model::<PartialUser>()
                        .all(&db)
                        .await
                    else {
                        tracing::error!("[Worker] Failed to query users from database.");
                        return;
                    };

                    if bad_users.is_empty() {
                        tracing::info!("[Worker] No expired users found. Job finished.");
                        return;
                    }

                    let keys: Vec<String> = bad_users
                        .iter()
                        .map(|user| format!("setup:{}", user.employee_id))
                        .collect();

                    let exists: Vec<bool> = match redis_conn.exists(keys).await {
                        Ok(values) => values,
                        Err(e) => {
                            tracing::error!("[Worker] Redis exists error: {}", e);
                            return;
                        }
                    };

                    let expired_user_ids: Vec<i32> = bad_users
                        .into_iter()
                        .zip(exists.into_iter())
                        .filter_map(|(user, is_exist)| {
                            if is_exist {
                                tracing::info!(
                                    "[Worker] Skipping user {} (still has setup token)",
                                    user.id
                                );
                                None
                            } else {
                                tracing::info!(
                                    "[Worker] Resetting user {} status (expired)",
                                    user.id
                                );
                                Some(user.id)
                            }
                        })
                        .collect();

                    if !expired_user_ids.is_empty() {
                        tracing::info!(
                            "[Worker] Resetting status for users: {:?}",
                            expired_user_ids
                        );

                        let update_result = users::Entity::update_many()
                            .col_expr(
                                users::Column::AccountStatus,
                                Expr::value(AccountStatus::PendingVerification),
                            )
                            .filter(users::Column::Id.is_in(expired_user_ids))
                            .exec(&db)
                            .await;

                        match update_result {
                            Ok(res) => tracing::info!(
                                "[Worker Successfully reset {} users.]",
                                res.rows_affected
                            ),
                            Err(e) => {
                                tracing::error!(
                                    "[Worker] Encounter error when bulk update users: {}",
                                    e
                                )
                            }
                        }
                    } else {
                        tracing::info!("[Worker] No users needed a reset. Job finished.")
                    }
                })
            })
            .unwrap(),
        )
        .await
    {
        Ok(sch) => sch,
        Err(e) => {
            tracing::error!("[Worker] Failed to add cron for register setup: {e}");
            return;
        }
    };

    match sched.start().await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("[Worker] Failed to start scheduler: {e}")
        }
    }
}
