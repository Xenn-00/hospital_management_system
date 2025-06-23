use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Employees::Table)
                    .add_column(ColumnDef::new(Employees::CreatedBy).integer().null())
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .from_tbl(Employees::Table)
                            .from_col(Employees::CreatedBy)
                            .to_tbl(Employees::Table)
                            .to_col(Employees::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Employees::Table)
                    .drop_column(Employees::CreatedBy)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Employees {
    Table,
    Id,
    CreatedBy,
}
