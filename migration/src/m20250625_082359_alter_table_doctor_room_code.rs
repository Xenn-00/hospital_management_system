use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Doctors::Table)
                    .drop_column(Doctors::RoomCode)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Doctors::Table)
                    .add_column(ColumnDef::new(Doctors::RoomCode).string().not_null())
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .name("fk_doctors_room_id")
                            .from_tbl(Doctors::Table)
                            .from_col(Doctors::RoomCode)
                            .to_tbl(Rooms::Table)
                            .to_col(Rooms::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Doctors {
    Table,
    RoomCode,
}

#[derive(DeriveIden)]
enum Rooms {
    Table,
    Code,
}
