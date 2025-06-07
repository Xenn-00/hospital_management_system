use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250527_141001_create_table_departments::Departments,
    m20250528_052903_create_table_room::Rooms,
};
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_051911_create_table_polyclinic"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Polyclinic::Table)
                    .if_not_exists()
                    .col(pk_auto(Polyclinic::Id))
                    .col(string(Polyclinic::Code).unique_key())
                    .col(string(Polyclinic::RoomCode))
                    .col(string(Polyclinic::Name))
                    .col(string_null(Polyclinic::Description))
                    .col(string(Polyclinic::DepartmentCode))
                    .col(timestamp(Polyclinic::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_polyclinic_department_id")
                            .from(Polyclinic::Table, Polyclinic::DepartmentCode)
                            .to(Departments::Table, Departments::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_polyclinic_room_code")
                            .from(Polyclinic::Table, Polyclinic::RoomCode)
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
            .drop_table(Table::drop().table(Polyclinic::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Polyclinic {
    Table,
    Id,
    Code,
    RoomCode, // 1 polyclinic can have multiple rooms
    Name,
    Description,
    DepartmentCode,
    CreatedAt,
}
