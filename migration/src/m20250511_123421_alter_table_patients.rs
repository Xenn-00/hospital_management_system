use sea_orm_migration::prelude::*;

use crate::m20250509_070014_create_patient_table::Patients;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250511_123421_alter_table_patients"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Patients::Table)
                    .modify_column(ColumnDef::new(Patients::KnownAllergies).null())
                    .to_owned(),
            )
            .await
    }
}
