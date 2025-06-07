use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

// use crate::m20250527_112721_create_table_employees::Employees;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250527_141001_create_table_departments"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Departments::Table)
                    .if_not_exists()
                    .col(pk_auto(Departments::Id))
                    .col(string(Departments::Code).unique_key())
                    .col(string(Departments::Name))
                    .col(string_null(Departments::Description))
                    .col(
                        enumeration(
                            Departments::DepartmentCategory,
                            Alias::new("department_category"),
                            DepartmentCategory::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(integer(Departments::HeadId))
                    .col(
                        enumeration(
                            Departments::Status,
                            Alias::new("status"),
                            DepartmentStatus::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(timestamp(Departments::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Departments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Departments {
    Table,
    Id,
    Code,
    Name,
    Description,
    DepartmentCategory,
    HeadId,
    Status,
    CreatedAt,
}

#[derive(DeriveIden, EnumIter)]
pub enum DepartmentCategory {
    Administrative,
    Clinical,
    Support,
    Igd,
    Emergency,
    Procurement,
    Finance,
    HumanResources,
    InformationTechnology,
    Nursing,
    Laboratory,
}

#[derive(DeriveIden, EnumIter)]
enum DepartmentStatus {
    Active,
    Inactive,
}
