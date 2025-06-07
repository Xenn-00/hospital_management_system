pub use sea_orm_migration::prelude::*;

mod m20250508_094052_create_patient_visit_intent_table;
mod m20250509_061644_create_queue_ticket_table;
mod m20250509_070014_create_patient_table;
mod m20250511_121632_alter_table_queue_ticket;
mod m20250511_123421_alter_table_patients;
mod m20250512_050855_alter_patients_table;
mod m20250521_071832_create_referral_documents_table;
mod m20250521_094041_alter_patients_visit_intent_table;
mod m20250527_112721_create_table_employees;
mod m20250527_114154_create_user_table;
mod m20250527_141001_create_table_departments;
mod m20250528_051911_create_table_polyclinic;
mod m20250528_052514_create_table_doctors;
mod m20250528_052903_create_table_room;
mod m20250528_054209_create_table_nurses;
mod m20250528_055359_create_table_position_title;
mod m20250528_060315_create_table_employee_position;
mod m20250528_061735_create_table_doctor_schedule;
mod m20250528_062543_create_table_nurse_polyclinic_assignment;
mod m20250528_144437_alter_table_department;
mod m20250529_023225_alter_table_department_head_id_null;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250509_070014_create_patient_table::Migration),
            Box::new(m20250508_094052_create_patient_visit_intent_table::Migration),
            Box::new(m20250509_061644_create_queue_ticket_table::Migration),
            Box::new(m20250511_121632_alter_table_queue_ticket::Migration),
            Box::new(m20250511_123421_alter_table_patients::Migration),
            Box::new(m20250512_050855_alter_patients_table::Migration),
            Box::new(m20250521_071832_create_referral_documents_table::Migration),
            Box::new(m20250521_094041_alter_patients_visit_intent_table::Migration),
            Box::new(m20250527_141001_create_table_departments::Migration),
            Box::new(m20250527_112721_create_table_employees::Migration),
            Box::new(m20250527_114154_create_user_table::Migration),
            Box::new(m20250528_052903_create_table_room::Migration),
            Box::new(m20250528_051911_create_table_polyclinic::Migration),
            Box::new(m20250528_052514_create_table_doctors::Migration),
            Box::new(m20250528_054209_create_table_nurses::Migration),
            Box::new(m20250528_055359_create_table_position_title::Migration),
            Box::new(m20250528_060315_create_table_employee_position::Migration),
            Box::new(m20250528_061735_create_table_doctor_schedule::Migration),
            Box::new(m20250528_062543_create_table_nurse_polyclinic_assignment::Migration),
            Box::new(m20250529_023225_alter_table_department_head_id_null::Migration),
            Box::new(m20250528_144437_alter_table_department::Migration),
        ]
    }
}
