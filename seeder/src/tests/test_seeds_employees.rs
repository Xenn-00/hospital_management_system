use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{seeds_departments::seed_departments, seeds_employees::seeds_employees},
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_employees_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        let result = seeds_employees(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding employees failed: {:?}",
            result.err()
        );

        let employee_count = entity::employees::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count employees");

        assert!(
            employee_count > 0,
            "Employees should be seeded successfully",
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction"); //cleanup
    })
    .await;
}

#[tokio::test]
async fn test_seeds_employees_skip() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;
        ctx.seed_departments().await;
        ctx.seed_employees().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");

        let before = entity::employees::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count employees");

        let _ = seeds_employees(&txn).await;

        let after = entity::employees::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count employees");

        assert!(before == after, "Employees seed should be skipped");

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}
