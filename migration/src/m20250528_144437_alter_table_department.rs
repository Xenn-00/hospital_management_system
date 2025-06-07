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
                    .table(Departments::Table)
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .from_tbl(Departments::Table)
                            .from_col(Departments::HeadId)
                            .to_tbl(Employees::Table)
                            .to_col(Employees::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Departments::Table)
                    .drop_foreign_key("fk_departments_head_id")
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Departments {
    Table,
    HeadId,
}

#[derive(DeriveIden)]
enum Employees {
    Table,
    Id,
}
