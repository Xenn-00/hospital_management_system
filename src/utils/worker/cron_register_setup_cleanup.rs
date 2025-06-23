use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::{Duration, Utc};
use entity::users::{self, AccountStatus};
use redis::AsyncCommands;
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tokio_cron_scheduler::{Job, JobScheduler};

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

                    let mut user_ids_to_reset = Vec::new();

                    for user in bad_users {
                        let cache_key = format!("setup:{}", user.employee_id);
                        let exist = match redis_conn.exists(&cache_key).await {
                            Ok(true) => true,
                            Ok(false) => false,
                            Err(e) => {
                                tracing::error!("[Worker] Redis error on key {}: {}", cache_key, e);
                                return;
                            }
                        };

                        if exist {
                            tracing::info!(
                                "[Worker] Skipping user {} (still has setup token)",
                                user.id
                            );
                            continue;
                        }

                        tracing::info!("[Worker] Resetting user {} status (expired)", user.id);
                        user_ids_to_reset.push(user.id);
                    }

                    if !user_ids_to_reset.is_empty() {
                        tracing::info!(
                            "[Worker] Resetting status for users: {:?}",
                            user_ids_to_reset
                        );

                        let update_result = users::Entity::update_many()
                            .col_expr(
                                users::Column::AccountStatus,
                                Expr::value(AccountStatus::PendingVerification),
                            )
                            .filter(users::Column::Id.is_in(user_ids_to_reset))
                            .exec(&db)
                            .await;

                        match update_result {
                            Ok(res) => tracing::info!(
                                "[Worker] Successfully reset {} users.",
                                res.rows_affected
                            ),
                            Err(e) => {
                                tracing::error!("[Worker] Failed to bulk update users: {}", e)
                            }
                        }
                    } else {
                        tracing::info!("[Worker] No users needed a reset. Job finished.");
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
