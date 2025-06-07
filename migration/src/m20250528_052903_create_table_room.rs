use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_052903_create_table_room"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Rooms::Table)
                    .if_not_exists()
                    .col(pk_auto(Rooms::Id))
                    .col(string(Rooms::Code).unique_key())
                    .col(string(Rooms::RoomType))
                    .col(string(Rooms::Status))
                    .col(timestamp(Rooms::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Rooms::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Rooms {
    Table,
    Id,
    Code,
    RoomType,
    Status,
    CreatedAt,
}
