use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250509_070014_create_patient_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Patients::Table)
                    .if_not_exists()
                    .col(pk_auto(Patients::Id))
                    .col(string(Patients::Name))
                    .col(date(Patients::DateOfBirth))
                    .col(string(Patients::NationalId))
                    .col(string_null(Patients::BPJSNumber))
                    .col(
                        enumeration(Patients::Gender, Alias::new("gender"), Gender::iter())
                            .string()
                            .not_null(),
                    )
                    .col(string(Patients::EmergencyContactName))
                    .col(string(Patients::EmergencyContactPhone))
                    .col(string(Patients::EmergencyContactRelationship))
                    .col(
                        enumeration(
                            Patients::BloodType,
                            Alias::new("blood_type"),
                            BloodType::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(string(Patients::KnownAllergies))
                    .col(timestamp(Patients::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Patients::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-patients_name")
                    .table(Patients::Table)
                    .col(Patients::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-patients_national_id")
                    .table(Patients::Table)
                    .col(Patients::NationalId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-patients_name").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-patients_national_id").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Patients::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden, EnumIter)]
pub enum Gender {
    #[iden = "Male"]
    Male,
    #[iden = "Female"]
    Female,
}

#[derive(Iden, EnumIter)]
#[iden = "blood_type"]
pub enum BloodType {
    #[iden = "A+"]
    APlus,
    #[iden = "A-"]
    AMinus,
    #[iden = "B+"]
    BPlus,
    #[iden = "B-"]
    BMinus,
    #[iden = "AB+"]
    ABPlus,
    #[iden = "AB-"]
    ABMinus,
    #[iden = "O+"]
    OPlus,
    #[iden = "O-"]
    OMinus,
}

#[derive(DeriveIden)]
pub enum Patients {
    Table,
    Id,
    Name,
    DateOfBirth,
    NationalId,
    BPJSNumber,
    Gender,
    EmergencyContactName,
    EmergencyContactPhone,
    EmergencyContactRelationship,
    BloodType,
    KnownAllergies,
    CreatedAt,
    UpdatedAt,
}
