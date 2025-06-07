use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250527_112721_create_table_employees::Employees,
    m20250528_051911_create_table_polyclinic::Polyclinic,
    m20250528_052903_create_table_room::Rooms,
};
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_052514_create_table_doctors"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Doctors::Table)
                    .if_not_exists()
                    .col(pk_auto(Doctors::Id))
                    .col(string(Doctors::Name))
                    .col(integer(Doctors::EmployeeId).unique_key())
                    .col(string(Doctors::Specialization))
                    .col(string(Doctors::LicenseNumber).unique_key())
                    .col(string(Doctors::RoomCode))
                    .col(integer(Doctors::PolyclinicId))
                    .col(timestamp(Doctors::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_doctors_employee_id")
                            .from(Doctors::Table, Doctors::EmployeeId)
                            .to(Employees::Table, Employees::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_doctors_polyclinic_id")
                            .from(Doctors::Table, Doctors::PolyclinicId)
                            .to(Polyclinic::Table, Polyclinic::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_doctors_room_id")
                            .from(Doctors::Table, Doctors::RoomCode)
                            .to(Rooms::Table, Rooms::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Doctors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Doctors {
    Table,
    Id,
    Name,
    EmployeeId,
    Specialization,
    LicenseNumber,
    RoomCode,
    PolyclinicId,
    CreatedAt,
}
