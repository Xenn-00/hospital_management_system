use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(DepartmentRoles::Table)
                    .if_not_exists()
                    .col(pk_auto(DepartmentRoles::Id))
                    .col(string(DepartmentRoles::DepartmentCode))
                    .col(integer(DepartmentRoles::RoleId))
                    .col(timestamp(DepartmentRoles::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(DepartmentRoles::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_department_role_department_code")
                            .from(DepartmentRoles::Table, DepartmentRoles::DepartmentCode)
                            .to(Departments::Table, Departments::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_department_role_role_id")
                            .from(DepartmentRoles::Table, DepartmentRoles::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(DepartmentRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DepartmentRoles {
    Table,
    Id,
    DepartmentCode,
    RoleId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Departments {
    Table,
    Code,
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
}
