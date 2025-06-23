use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Employees::Table)
                    .add_column(ColumnDef::new(Employees::Gender).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Employees::Table)
                    .drop_column(Employees::Gender)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Employees {
    Table,
    Gender,
}
