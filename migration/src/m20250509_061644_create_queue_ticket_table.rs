use sea_orm_migration::{
    prelude::*,
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

use crate::m20250508_094052_create_patient_visit_intent_table::PatientsVisitIntent;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250509_061644_create_queue_ticket_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(QueueTicket::Table)
                    .if_not_exists()
                    .col(pk_auto(QueueTicket::Id))
                    .col(integer(QueueTicket::VisitIntentId).unique_key())
                    .col(integer(QueueTicket::QueueNumber))
                    .col(
                        enumeration(
                            QueueTicket::QueueType,
                            Alias::new("queue_type"),
                            QueueType::iter(),
                        )
                        .string()
                        .not_null(),
                    )
                    .col(
                        enumeration(QueueTicket::Status, Alias::new("status"), Status::iter())
                            .string()
                            .not_null(),
                    )
                    .col(timestamp(QueueTicket::CalledAt).default(Expr::current_timestamp()))
                    .col(timestamp(QueueTicket::DoneAt).default(Expr::current_timestamp()))
                    .col(timestamp(QueueTicket::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-queue_ticket-visit_intent_id")
                            .from(QueueTicket::Table, QueueTicket::VisitIntentId)
                            .to(PatientsVisitIntent::Table, PatientsVisitIntent::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(QueueTicket::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum QueueTicket {
    Table,
    Id,
    VisitIntentId,
    QueueNumber,
    QueueType,
    Status,
    CalledAt,
    DoneAt,
    CreatedAt,
}

#[derive(Iden, EnumIter)]
pub enum QueueType {
    COMMON,
    BPJS,
}

#[derive(Iden, EnumIter)]
pub enum Status {
    WAITING,
    CALLED,
    DONE,
    SKIPPED,
    CANCELLED,
}
