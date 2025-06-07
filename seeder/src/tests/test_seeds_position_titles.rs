use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{seeds_departments::seed_departments, seeds_positions_title::seeds_positions_title},
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_position_titles_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        let result = seeds_positions_title(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding position titles failed: {:?}",
            result.err()
        );

        let position_count = entity::position_titles::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count position titles");

        assert!(
            position_count > 0,
            "Position titles should be seeded successfully"
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}
