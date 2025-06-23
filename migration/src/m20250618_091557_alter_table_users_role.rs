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
                    .add_column(ColumnDef::new(Users::RoleId).integer().not_null())
                    .drop_column(Users::Role)
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .from_tbl(Users::Table)
                            .from_col(Users::RoleId)
                            .to_tbl(Role::Table)
                            .to_col(Role::Id),
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
                    .add_column(ColumnDef::new(Users::Role).string().not_null())
                    .drop_column(Users::RoleId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Role,
    RoleId,
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
}
