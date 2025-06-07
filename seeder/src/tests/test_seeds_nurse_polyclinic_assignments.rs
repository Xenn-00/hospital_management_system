use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{
        seeds_departments::seed_departments, seeds_doctors::seeds_doctors,
        seeds_employees::seeds_employees,
        seeds_nurse_polyclinic_assignments::seeds_nurse_polyclinic_assignments,
        seeds_nurses::seeds_nurses, seeds_polyclinic::seeds_polyclinic, seeds_rooms::seed_rooms,
    },
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_nurse_polyclinic_assignments_success() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_rooms(&txn).await.expect("Failed to seed rooms");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        seeds_employees(&txn)
            .await
            .expect("Failed to seed employees");
        seeds_polyclinic(&txn)
            .await
            .expect("Failed to seed polyclincs");
        seeds_doctors(&txn).await.expect("Failed to seed doctors");
        seeds_nurses(&txn).await.expect("Failed to seed nurses");
        let result = seeds_nurse_polyclinic_assignments(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding nurse assignment failed: {:?}",
            result.err()
        );

        let nurse_count = entity::nurses_polyclinic_assignments::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count nurse assignment");

        assert!(
            nurse_count > 0,
            "nurse assignment should be seeded successfully",
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction"); //cleanup
    })
    .await;
}
