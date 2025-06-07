use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250527_141001_create_table_departments::Departments;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250527_112721_create_table_employees"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Employees::Table)
                    .if_not_exists()
                    .col(pk_auto(Employees::Id))
                    .col(string(Employees::FullName))
                    .col(string_null(Employees::NIP).unique_key())
                    .col(string(Employees::Email).unique_key())
                    .col(string(Employees::Phone).unique_key())
                    .col(date(Employees::BirthDate))
                    .col(date(Employees::HireDate))
                    .col(string(Employees::Address))
                    .col(
                        enumeration(
                            Employees::EmploymentStatus,
                            Alias::new("employment_status"),
                            EmployementStatus::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(string(Employees::DepartmentCode))
                    .col(timestamp(Employees::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_employees_department_id")
                            .from(Employees::Table, Employees::DepartmentCode)
                            .to(Departments::Table, Departments::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_employees_full_name")
                    .table(Employees::Table)
                    .col(Employees::FullName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(Index::drop().name("idx_employees_full_name").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Employees::Table).to_owned())
            .await
            .map_err(|e| e.into())
    }
}

#[derive(DeriveIden)]
pub enum Employees {
    Table,
    Id,
    FullName,
    NIP,
    Email,
    Phone,
    BirthDate,
    Address,
    HireDate,
    EmploymentStatus,
    DepartmentCode,
    CreatedAt,
}

#[derive(Iden, EnumIter)]
pub enum EmployementStatus {
    Permanent,
    Contract,
    Internship,
    Resigned,
    Terminated,
}
