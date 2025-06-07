use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250527_141001_create_table_departments::Departments;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250528_052903_create_table_position_title"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(PositionTitles::Table)
                    .if_not_exists()
                    .col(pk_auto(PositionTitles::Id))
                    .col(string(PositionTitles::Title).unique_key())
                    .col(string_null(PositionTitles::Description))
                    .col(string(PositionTitles::DepartmentCode))
                    .col(timestamp(PositionTitles::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(PositionTitles::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_position_titles_department_id")
                            .from(PositionTitles::Table, PositionTitles::DepartmentCode)
                            .to(Departments::Table, Departments::Code)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(PositionTitles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PositionTitles {
    Table,
    Id,
    Title,
    Description,
    DepartmentCode,
    CreatedAt,
    UpdatedAt,
}
