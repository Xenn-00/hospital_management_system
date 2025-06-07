use sea_orm::{EntityTrait, PaginatorTrait, TransactionTrait};

use crate::{
    seeds::{
        seeds_departments::seed_departments, seeds_employees::seeds_employees,
        seeds_employees_position::seeds_employees_position,
        seeds_positions_title::seeds_positions_title,
    },
    tests::context::{init_db_lock, with_db_lock, TestContext},
};

#[tokio::test]
async fn test_seeds_employees_position() {
    init_db_lock();

    with_db_lock(|| async {
        let ctx = TestContext::new().await;
        ctx.reset().await;

        let txn = ctx.db.begin().await.expect("Failed to begin transaction");
        seed_departments(&txn)
            .await
            .expect("Failed to seed departments");
        seeds_employees(&txn)
            .await
            .expect("Failed to seed employees");
        seeds_positions_title(&txn)
            .await
            .expect("Failed to seed position titles");

        let result = seeds_employees_position(&txn).await;

        assert!(
            result.is_ok(),
            "Seeding employees position failed: {:?}",
            result.err()
        );

        let employees_position_count = entity::employee_position::Entity::find()
            .count(&txn)
            .await
            .expect("Failed to count employees position");

        assert!(
            employees_position_count > 0,
            "employees position should be seeded successfully",
        );

        ctx.reset().await;

        txn.rollback()
            .await
            .expect("Failed to rollback transaction"); //cleanup
    })
    .await
}
