use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::CreatedBy).integer().null())
                    .add_column(ColumnDef::new(Users::UpdatedBy).integer().null())
                    .add_column(ColumnDef::new(Users::DeletedBy).integer().null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Users::Table)
                            .from_col(Users::CreatedBy)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Users::Table)
                            .from_col(Users::UpdatedBy)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Users::Table)
                            .from_col(Users::DeletedBy)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::CreatedBy)
                    .drop_column(Users::UpdatedBy)
                    .drop_column(Users::DeletedBy)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    CreatedBy,
    UpdatedBy,
    DeletedBy,
}
