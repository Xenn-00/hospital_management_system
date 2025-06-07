use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{
        seeds_departments::seed_departments, seeds_polyclinic::seeds_polyclinic,
        seeds_rooms::seed_rooms,
    },
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_polyclinics_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_rooms(&txn).await.expect("Failed to seed rooms");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        let result = seeds_polyclinic(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding polyclinics failed: {:?}",
            result.err()
        );

        let polyclinics_count = entity::polyclinic::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count polyclinics");

        assert!(
            polyclinics_count > 0,
            "Polyclinics should be seeded successfully",
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction"); //cleanup
    })
    .await;
}

#[tokio::test]
#[should_panic(expected = "❌ Not enough available rooms to assign to polyclinics!")]
async fn test_polyclinic_seed_fail_not_enough_rooms() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;
        ctx.seed_rooms(10).await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        let _ = seeds_polyclinic(&txn).await;

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction");
    })
    .await;
}
