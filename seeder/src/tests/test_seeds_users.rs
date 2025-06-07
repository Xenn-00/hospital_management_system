use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{
        seeds_departments::seed_departments, seeds_employees::seeds_employees,
        seeds_users::seeds_users,
    },
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_users_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        seeds_employees(&txn).await.expect("Failed to seed users");
        let result = seeds_users(&txn).await;

        assert!(result.is_ok(), "Seeding users failed: {:?}", result.err());

        let users_count = entity::user::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count users");

        assert!(users_count > 0, "Users should be seeded successfully",);

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction"); //cleanup
    })
    .await;
}
