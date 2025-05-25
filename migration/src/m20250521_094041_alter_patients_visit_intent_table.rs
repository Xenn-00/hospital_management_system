use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250521_094041_alter_patients_visit_intent_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(PatientsVisitIntent::Table)
                    .drop_column(PatientsVisitIntent::ReferralDocumentUrl)
                    .add_column(
                        ColumnDef::new(PatientsVisitIntent::ReferralDocumentId)
                            .integer()
                            .null(),
                    )
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .from_tbl(PatientsVisitIntent::Table)
                            .from_col(PatientsVisitIntent::ReferralDocumentId)
                            .to_tbl(ReferralDocuments::Table)
                            .to_col(ReferralDocuments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PatientsVisitIntent::Table)
                    .drop_column(PatientsVisitIntent::ReferralDocumentId)
                    .add_column(
                        ColumnDef::new(PatientsVisitIntent::ReferralDocumentUrl)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum PatientsVisitIntent {
    Table,
    ReferralDocumentUrl,
    ReferralDocumentId,
}

#[derive(Iden)]
enum ReferralDocuments {
    Table,
    Id,
}
