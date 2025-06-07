use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250527_112721_create_table_employees::Employees,
    m20250527_141001_create_table_departments::Departments,
    m20250528_055359_create_table_position_title::PositionTitles,
};
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_060315_create_table_employee_position"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(EmployeePosition::Table)
                    .if_not_exists()
                    .col(pk_auto(EmployeePosition::Id))
                    .col(integer(EmployeePosition::EmployeeId))
                    .col(integer(EmployeePosition::PositionTitleId))
                    .col(string(EmployeePosition::DepartmentCode))
                    .col(timestamp(EmployeePosition::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(EmployeePosition::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_employee_position_employee_id")
                            .from(EmployeePosition::Table, EmployeePosition::EmployeeId)
                            .to(Employees::Table, Employees::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_employee_position_position_title_id")
                            .from(EmployeePosition::Table, EmployeePosition::PositionTitleId)
                            .to(PositionTitles::Table, PositionTitles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_employee_position_department_code")
                            .from(EmployeePosition::Table, EmployeePosition::DepartmentCode)
                            .to(Departments::Table, Departments::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(EmployeePosition::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum EmployeePosition {
    Table,
    Id,
    EmployeeId,
    PositionTitleId,
    DepartmentCode,
    CreatedAt,
    UpdatedAt,
}
