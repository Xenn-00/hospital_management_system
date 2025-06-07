use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250527_112721_create_table_employees::Employees,
    m20250528_051911_create_table_polyclinic::Polyclinic,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_054209_create_table_nurses"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Nurses::Table)
                    .if_not_exists()
                    .col(pk_auto(Nurses::Id))
                    .col(string(Nurses::Name))
                    .col(integer(Nurses::EmployeeId).unique_key())
                    .col(string(Nurses::LicenseNumber).unique_key())
                    .col(integer(Nurses::PolyclinicId))
                    .col(timestamp(Nurses::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nurses_employee_id")
                            .from(Nurses::Table, Nurses::EmployeeId)
                            .to(Employees::Table, Employees::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nurses_polyclinic_id")
                            .from(Nurses::Table, Nurses::PolyclinicId)
                            .to(Polyclinic::Table, Polyclinic::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Nurses::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Nurses {
    Table,
    Id,
    Name,
    EmployeeId,
    LicenseNumber,
    PolyclinicId,
    CreatedAt,
}
