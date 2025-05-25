use sea_orm_migration::{
    prelude::*,
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

use crate::{
    m20250508_094052_create_patient_visit_intent_table::PatientsVisitIntent,
    m20250509_070014_create_patient_table::Patients,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250521_071832_create_referral_documents_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(ReferralDocuments::Table)
                    .if_not_exists()
                    .col(pk_auto(ReferralDocuments::Id))
                    .col(integer(ReferralDocuments::PatientsId))
                    .col(integer(ReferralDocuments::VisitIntentId))
                    .col(string(ReferralDocuments::FileName))
                    .col(big_integer(ReferralDocuments::FileSize))
                    .col(
                        enumeration(
                            ReferralDocuments::Status,
                            Alias::new("status"),
                            Status::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(string(ReferralDocuments::ReferralDocumentURL))
                    .col(timestamp_null(ReferralDocuments::ScannedAt))
                    .col(timestamp(ReferralDocuments::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(ReferralDocuments::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-referral_documents-visit_intent_id")
                            .from(ReferralDocuments::Table, ReferralDocuments::VisitIntentId)
                            .to(PatientsVisitIntent::Table, PatientsVisitIntent::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-referral_documents-patients_id")
                            .from(ReferralDocuments::Table, ReferralDocuments::PatientsId)
                            .to(Patients::Table, Patients::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(ReferralDocuments::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ReferralDocuments {
    Table,
    Id,
    PatientsId,
    VisitIntentId,
    FileName,
    FileSize,
    Status,
    ReferralDocumentURL,
    ScannedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden, EnumIter)]
pub enum Status {
    WAITING,
    SCANNING,
    UPLOADED,
    FAILED,
}
