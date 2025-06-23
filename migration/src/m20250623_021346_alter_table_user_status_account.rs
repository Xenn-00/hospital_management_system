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
                    .table(Users::Table)
                    .drop_column(Users::IsActive)
                    .add_column(
                        ColumnDef::new(Users::AccountStatus)
                            .string()
                            .not_null()
                            .default("PENDING_VERIFICATION"),
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
                    .table(Users::Table)
                    .drop_column(Users::AccountStatus)
                    .add_column(ColumnDef::new(Users::IsActive).boolean().default(false))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    AccountStatus,
    IsActive,
}
