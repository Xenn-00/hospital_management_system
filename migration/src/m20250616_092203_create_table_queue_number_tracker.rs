use sea_orm_migration::{
    prelude::*,
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(QueueSequence::Table)
                    .if_not_exists()
                    .col(pk_auto(QueueSequence::Id))
                    .col(
                        enumeration(
                            QueueSequence::VisitType,
                            Alias::new("visit_type"),
                            VisitType::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(integer(QueueSequence::LastNumber).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(QueueSequence::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum QueueSequence {
    Table,
    Id,
    VisitType,
    LastNumber,
}

#[derive(Iden, EnumIter)]
enum VisitType {
    BPJS,
    COMMON,
    EMERGENCY,
    REFERRAL,
    OTHER,
}
