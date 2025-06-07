use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250527_112721_create_table_employees::Employees;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250527_114154_create_user_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(integer(User::EmployeeId).unique_key())
                    .col(string(User::Username).unique_key())
                    .col(string(User::Password))
                    .col(
                        enumeration(User::Role, Alias::new("role"), UserRole::iter())
                            .string()
                            .not_null(),
                    )
                    .col(timestamp_null(User::LastLogin))
                    .col(boolean(User::IsActive).default(true))
                    .col(timestamp(User::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_employees_id")
                            .from(User::Table, User::EmployeeId)
                            .to(Employees::Table, Employees::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    EmployeeId,
    Username,
    Password,
    Role,
    LastLogin,
    IsActive,
    CreatedAt,
}

#[derive(Iden, EnumIter)]
pub enum UserRole {
    SuperAdmin,    // Super Admin has all permissions, system owner
    Admin,         // Admin system, can manage users and settings
    HR,            // Human Resources, can manage employee records
    Finance,       // Finance department, can manage financial records
    Doctor,        // Doctor General Practitioner/Specialist, can manage medical records
    Nurse,         // Nurse, manage medical records, schedule appointments, etc
    Pharmacist,    // Pharmacist, manage prescriptions and medications
    LabTechnician, // Lab Technician, manage lab tests and results
    FrontOffice,   // Front Office, manage patient appointments and inquiries
    SupportStaff,  // cleaning, security, etc (minimum access)
}
