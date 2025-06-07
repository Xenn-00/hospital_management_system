use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::seeds_departments::seed_departments,
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_departements_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        let result = seed_departments(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding department failed: {:?}",
            result.err()
        );

        let department_count = entity::departments::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count departments");

        assert!(
            department_count > 0,
            "Departments should be seeded successfully"
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}

#[tokio::test]
async fn test_seeds_departments_skip() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;
        ctx.seed_departments().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");

        let before = entity::departments::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count departments");

        let _ = seed_departments(&txn).await;

        let after = entity::departments::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count departments");

        assert!(before == after, "Departments seed should be skipped");

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}
