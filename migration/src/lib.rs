pub use sea_orm_migration::prelude::*;

mod m20250508_094052_create_patient_visit_intent_table;
mod m20250509_061644_create_queue_ticket_table;
mod m20250509_070014_create_patient_table;
mod m20250511_121632_alter_table_queue_ticket;
mod m20250511_123421_alter_table_patients;
mod m20250512_050855_alter_patients_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250508_094052_create_patient_visit_intent_table::Migration),
            Box::new(m20250509_061644_create_queue_ticket_table::Migration),
            Box::new(m20250509_070014_create_patient_table::Migration),
            Box::new(m20250511_121632_alter_table_queue_ticket::Migration),
            Box::new(m20250511_123421_alter_table_patients::Migration),
            Box::new(m20250512_050855_alter_patients_table::Migration),
        ]
    }
}
