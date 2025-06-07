use entity::{polyclinic, rooms};
use log::info;
use sea_orm::{ColumnTrait, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait, QueryFilter};

use crate::helpers::generate_polyclinic;

pub async fn seeds_polyclinic(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    info!("🚀 Seeding polyclinics...");
    let count = polyclinic::Entity::find().count(txn).await?;

    if count > 0 {
        info!("⚠️  Polyclinics already seeded. Skipping...");
        return Ok(());
    }
    let polyclinics_list = vec![
        "Cardiology",
        "Neurology",
        "Pediatrics",
        "Orthopedics",
        "General Surgery",
        "Dermatology",
        "Psychiatry",
        "Radiology",
        "Gynecology",
        "Ophthalmology",
        "ENT",
        "Urology",
        "Gastroenterology",
        "Oncology",
        "Anesthesiology",
        "Pathology",
        "Internal Medicine",
        "Family Medicine",
    ];

    let available_rooms = rooms::Entity::find()
        .filter(rooms::Column::Status.eq("AVAILABLE".to_string()))
        .filter(rooms::Column::RoomType.eq("UNASSIGNED".to_string()))
        .all(txn)
        .await?;

    let mut rooms: Vec<String> = available_rooms.into_iter().map(|room| room.code).collect();

    // We need exactly 18 rooms for seeding polyclinics
    if rooms.len() < polyclinics_list.len() {
        panic!("❌ Not enough available rooms to assign to polyclinics!");
    }
    let (polyclinic_rooms, _) = rooms.split_at_mut(18);

    let polyclinic_room_codes: Vec<String> = polyclinic_rooms.to_vec();

    rooms::Entity::update_many()
        .filter(rooms::Column::Code.is_in(polyclinic_room_codes.clone()))
        .col_expr(
            rooms::Column::Status,
            sea_orm::sea_query::SimpleExpr::Value(sea_orm::Value::String(Some(Box::new(
                "OCCUPIED".to_string(),
            )))),
        )
        .col_expr(
            rooms::Column::RoomType,
            sea_orm::sea_query::SimpleExpr::Value(sea_orm::Value::String(Some(Box::new(
                "POLYCLINIC".to_string(),
            )))),
        )
        .exec(txn)
        .await?;

    let polyclinic_models = generate_polyclinic(polyclinics_list, polyclinic_rooms.to_vec());

    if polyclinic_models.is_empty() {
        panic!("❌ No polyclinics generated to seed!");
    }
    polyclinic::Entity::insert_many(polyclinic_models)
        .exec(txn)
        .await?;

    info!("✅ Polyclinics seeded successfully.");

    Ok(())
}
