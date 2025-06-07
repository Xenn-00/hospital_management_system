use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250528_051911_create_table_polyclinic::Polyclinic,
    m20250528_052514_create_table_doctors::Doctors,
};
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_061735_create_table_doctor_schedule"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(DoctorSchedules::Table)
                    .if_not_exists()
                    .col(pk_auto(DoctorSchedules::Id))
                    .col(integer(DoctorSchedules::DoctorId))
                    .col(integer(DoctorSchedules::PolyclinicId))
                    .col(string(DoctorSchedules::RoomCode))
                    .col(
                        enumeration(
                            DoctorSchedules::DayOfWeek,
                            Alias::new("day_of_week"),
                            DayOfWeek::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(time(DoctorSchedules::StartTime).not_null())
                    .col(time(DoctorSchedules::EndTime).not_null())
                    .col(
                        enumeration(
                            DoctorSchedules::Status,
                            Alias::new("status"),
                            Status::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(timestamp(DoctorSchedules::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(DoctorSchedules::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_doctor_schedules_doctor_id")
                            .from(DoctorSchedules::Table, DoctorSchedules::DoctorId)
                            .to(Doctors::Table, Doctors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_doctor_schedules_polyclinic_id")
                            .from(DoctorSchedules::Table, DoctorSchedules::PolyclinicId)
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
            .drop_table(Table::drop().table(DoctorSchedules::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DoctorSchedules {
    Table,
    Id,
    DoctorId,
    PolyclinicId,
    RoomCode,
    DayOfWeek,
    StartTime,
    EndTime,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, EnumIter)]
enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(DeriveIden, EnumIter)]
enum Status {
    Active,
    Inactive,
}
