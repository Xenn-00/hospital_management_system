use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

use super::m20250509_070014_create_patient_table::Patients;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250508_094052_create_patient_visit_intent_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PatientsVisitIntent::Table)
                    .if_not_exists()
                    .col(pk_auto(PatientsVisitIntent::Id))
                    .col(integer(PatientsVisitIntent::PatientId))
                    .col(
                        enumeration(
                            PatientsVisitIntent::VisitType,
                            Alias::new("visit_type"),
                            VisitType::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(string_null(PatientsVisitIntent::ReferralDocumentUrl))
                    .col(
                        enumeration(
                            PatientsVisitIntent::Status,
                            Alias::new("status"),
                            Status::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(string(PatientsVisitIntent::ReferralValidated).default("false"))
                    .col(
                        timestamp(PatientsVisitIntent::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(PatientsVisitIntent::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-patients_visit_intent-patient_id")
                            .from(PatientsVisitIntent::Table, PatientsVisitIntent::PatientId)
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PatientsVisitIntent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PatientsVisitIntent {
    Table,
    Id,
    PatientId,
    VisitType,
    ReferralDocumentUrl,
    ReferralValidated,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden, EnumIter)]
pub enum VisitType {
    BPJS,
    COMMON,
    EMERGENCY,
    REFERRAL,
    OTHER,
}
#[derive(Iden, EnumIter)]
pub enum Status {
    PENDING,
    INPROGRESS,
    COMPLETED,
    CANCELLED,
}
