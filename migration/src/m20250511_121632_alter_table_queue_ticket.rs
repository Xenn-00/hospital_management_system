use sea_orm_migration::prelude::*;

use crate::m20250509_061644_create_queue_ticket_table::QueueTicket;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250511_121632_alter_table_queue_ticket"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(QueueTicket::Table)
                    .modify_column(ColumnDef::new(QueueTicket::CalledAt).timestamp().null())
                    .modify_column(ColumnDef::new(QueueTicket::DoneAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }
}
