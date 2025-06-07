use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250528_051911_create_table_polyclinic::Polyclinic,
    m20250528_054209_create_table_nurses::Nurses,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_062543_create_table_nurse_polyclinic_assignment"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(NursesPolyclinicAssignments::Table)
                    .if_not_exists()
                    .col(pk_auto(NursesPolyclinicAssignments::Id))
                    .col(integer(NursesPolyclinicAssignments::NurseId))
                    .col(integer(NursesPolyclinicAssignments::PolyclinicId))
                    .col(
                        timestamp(NursesPolyclinicAssignments::AssignedSince)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_null(NursesPolyclinicAssignments::AssignedUntil))
                    .col(string_null(NursesPolyclinicAssignments::Notes))
                    .col(
                        timestamp(NursesPolyclinicAssignments::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(NursesPolyclinicAssignments::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nurse_polyclinic_assignment_nurse_id")
                            .from(
                                NursesPolyclinicAssignments::Table,
                                NursesPolyclinicAssignments::NurseId,
                            )
                            .to(Nurses::Table, Nurses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nurse_polyclinic_assignment_polyclinic_id")
                            .from(
                                NursesPolyclinicAssignments::Table,
                                NursesPolyclinicAssignments::PolyclinicId,
                            )
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
            .drop_table(
                Table::drop()
                    .table(NursesPolyclinicAssignments::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum NursesPolyclinicAssignments {
    Table,
    Id,
    NurseId,
    PolyclinicId,
    AssignedSince,
    AssignedUntil,
    Notes,
    CreatedAt,
    UpdatedAt,
}
