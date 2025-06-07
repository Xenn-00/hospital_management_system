use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::seeds_rooms::seed_rooms,
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_rooms_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        let result = seed_rooms(&txn).await;
        assert!(result.is_ok(), "Seeding room failed: {:?}", result.err());

        let room_count = entity::rooms::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count Rooms");

        assert!(room_count > 0, "Rooms should be seeded successfully");

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}

#[tokio::test]
async fn test_seeds_rooms_skip() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;
        ctx.seed_rooms(50).await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");

        let before = entity::rooms::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count rooms");

        let _ = seed_rooms(&txn).await;

        let after = entity::rooms::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count rooms");

        assert!(before == after, "rooms seed should be skipped");

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}
