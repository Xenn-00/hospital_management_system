use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use log::warn;
use rand::prelude::*;
use std::collections::HashMap;

use entity::{
    departments, doctor_schedules, doctors, employee_position, employees, nurses,
    nurses_polyclinic_assignments, polyclinic, position_titles, rooms,
    user::{self, Role},
};
use sea_orm::ActiveValue::Set;

pub fn generate_department(department_list: Vec<&str>) -> Vec<departments::ActiveModel> {
    department_list
        .iter()
        .enumerate()
        .map(|(idx, &name)| {
            let code = format!("DPT{:02}", idx + 1);
            let department_category = name.replace(' ', "_").to_uppercase();
            departments::ActiveModel {
                code: Set(code),
                name: Set(name.to_string()),
                department_category: Set(department_category),
                description: Set(Some(format!(
                    "{} Department for regional hospital of xxx",
                    name
                ))),
                head_id: Set(None), // Assuming head_id can be null
                status: Set("ACTIVE".to_string()),
                ..Default::default()
            }
        })
        .collect()
}

pub fn generate_employee(n: usize) -> Vec<employees::ActiveModel> {
    let mut rng = rand::rng();

    // Department distribution
    let dept_distribution = vec![
        ("DPT01", 15), // Administrative
        ("DPT02", 2),  // Human Resource
        ("DPT03", 5),  // Finance
        ("DPT04", 5),  // IT
        ("DPT05", 35), // Clinical
        ("DPT06", 5),  // Emergency
        ("DPT07", 5),  // Procurement
        ("DPT08", 15), // Nursing
        ("DPT09", 8),  // Laboratory
        ("DPT10", 5),  // Support
    ];

    // Status distribution
    let status_distribution = vec![
        ("Permanent", 50),
        ("Contract", 30),
        ("Internship", 15),
        ("Resigned", 3),
        ("Terminated", 2),
    ];

    let expand_dist = |dist: &Vec<(&str, usize)>| -> Vec<String> {
        dist.iter()
            .flat_map(|(v, percent)| std::iter::repeat((*v).to_string()).take(percent * n / 100))
            .collect()
    };

    let mut dept_pool = expand_dist(&dept_distribution);
    let mut status_pool = expand_dist(&status_distribution);

    dept_pool.shuffle(&mut rng);
    status_pool.shuffle(&mut rng);

    (0..n)
        .map(|i| {
            let binding_dpt = "DPT01".to_string();
            let dept_code = dept_pool.get(i % dept_pool.len()).unwrap_or(&binding_dpt);
            let binding_status = "Permanent".to_string();
            let status = status_pool
                .get(i % status_pool.len())
                .unwrap_or(&binding_status);

            let birth_year = rng.random_range(1970..=1995);
            let birth_month = rng.random_range(1..=12);
            let hire_year = rng.random_range(2015..=2023);

            employees::ActiveModel {
                full_name: Set(format!("Employee {}", i + 1)),
                email: Set(format!("employee{}@test.com", i + 1)),
                nip: Set(Some(format!(
                    "{:04}{:02}{:02}2000{:02}{}{:03}",
                    birth_year,
                    birth_month,
                    rng.random_range(1..=28),
                    birth_month,
                    rng.random_range(1..=2),
                    i + 1
                ))),
                phone: Set(format!("+12345678{}90", i + 1)),
                address: Set(format!("Address {}, City, Country", i + 1)),
                department_code: Set(dept_code.to_string()),
                birth_date: Set(NaiveDate::from_ymd_opt(birth_year, 1, 1).unwrap()),
                employment_status: Set(status.to_string()),
                hire_date: Set(NaiveDate::from_ymd_opt(hire_year, 1, 1).unwrap()),
                ..Default::default()
            }
        })
        .collect()
}

pub fn generate_doctor(
    n: i32,
    mut employees_ids: Vec<i32>,
    mut rooms: Vec<String>,
    polyclinics_ids_and_names: Vec<(i32, String)>,
) -> Vec<doctors::ActiveModel> {
    let mut rng = rand::rng();
    let mut doctors = Vec::new();
    for i in 1..=n {
        if employees_ids.is_empty() || rooms.is_empty() || polyclinics_ids_and_names.is_empty() {
            warn!("❌ Not enough data to generate more doctors!");
            break;
        }
        let license_number = format!("LN{:04}", i);
        let employee_id = *employees_ids.choose(&mut rng).unwrap();
        employees_ids.retain(|&x| x != employee_id);

        let room = rooms.choose(&mut rng).unwrap().clone();
        rooms.retain(|r| r != &room);

        let (polyclinic_id, specialization) =
            polyclinics_ids_and_names.choose(&mut rng).unwrap().clone();

        doctors.push(doctors::ActiveModel {
            name: Set(format!("Doctor {}", i)),
            employee_id: Set(employee_id),
            specialization: Set(specialization),
            license_number: Set(license_number),
            room_code: Set(room),
            polyclinic_id: Set(polyclinic_id),
            ..Default::default()
        });
    }
    doctors
}

pub fn generate_nurses(
    n: i32,
    mut employees_ids: Vec<i32>,
    filled_polyclinics_ids: Vec<i32>,
) -> Vec<nurses::ActiveModel> {
    let mut rng = rand::rng();
    let mut nurses = Vec::new();
    for i in 1..=n {
        if employees_ids.is_empty() || filled_polyclinics_ids.is_empty() {
            warn!("❌ Not enough data to generate more nurses!");
            break;
        }
        let license_number = format!("LN{:04}", i);
        let employee_id = *employees_ids.choose(&mut rng).unwrap();
        employees_ids.retain(|&x| x != employee_id);

        let poly_id = *filled_polyclinics_ids.choose(&mut rng).unwrap();

        nurses.push(nurses::ActiveModel {
            name: Set(format!("Nurses {}", i)),
            employee_id: Set(employee_id),
            license_number: Set(license_number),
            polyclinic_id: Set(poly_id),
            ..Default::default()
        });
    }

    nurses
}

pub fn generate_polyclinic(
    polyclinics_list: Vec<&str>,
    mut rooms: Vec<String>,
) -> Vec<polyclinic::ActiveModel> {
    let mut rng = rand::rng();
    polyclinics_list
        .into_iter()
        .enumerate()
        .map(|(idx, name)| {
            let room = rooms.choose(&mut rng).unwrap().clone();
            rooms.retain(|r| r != &room);
            polyclinic::ActiveModel {
                name: Set(name.to_string()),
                description: Set(Some(format!("Polyclinic specializing in {}", name))),
                department_code: Set("DPT05".to_string()), // Assuming all polyclinics belong to the Clinical Department
                code: Set(format!("PC{:02}", idx + 1)),    // Random code for polyclinic
                room_code: Set(room),                      // Random room code
                ..Default::default()
            }
        })
        .collect()
}

pub fn generate_room(n: usize) -> Vec<rooms::ActiveModel> {
    (1..=n)
        .map(|i| rooms::ActiveModel {
            code: Set(format!("RC{:02}", i)),
            status: Set("AVAILABLE".to_string()),
            room_type: Set("UNASSIGNED".to_string()),
            ..Default::default()
        })
        .collect()
}

pub fn generate_position_title(
    positions: Vec<(&'static str, &'static str, &'static str)>,
    dept_map: HashMap<String, String>,
) -> Vec<position_titles::ActiveModel> {
    let mut models = Vec::new();
    for (title, desc, dept_name) in positions {
        let code = dept_map
            .get(dept_name)
            .expect(&format!("Department not found: {}", dept_name));

        models.push(position_titles::ActiveModel {
            title: Set(title.to_string()),
            description: Set(Some(desc.to_string())),
            department_code: Set(code.clone()),
            ..Default::default()
        });
    }

    models
}

pub fn generate_employee_position(
    employees_id_and_dept: Vec<(i32, String)>,
    position_title_id_and_dept: Vec<(i32, String)>,
) -> Vec<employee_position::ActiveModel> {
    let mut rng = rand::rng();

    let mut title_map: HashMap<String, Vec<&(i32, String)>> = HashMap::new();
    for title in &position_title_id_and_dept {
        title_map.entry(title.1.clone()).or_default().push(title);
    }

    employees_id_and_dept
        .into_iter()
        .filter_map(|emp| {
            if let Some(titles) = title_map.get(&emp.1) {
                let selected_title = titles.choose(&mut rng)?;
                Some(employee_position::ActiveModel {
                    employee_id: Set(emp.0),
                    department_code: Set(emp.1.clone()),
                    position_title_id: Set(selected_title.0),
                    ..Default::default()
                })
            } else {
                None
            }
        })
        .collect()
}

pub struct Doctor {
    pub id: i32,
    pub poly_id: i32,
    pub room_code: String,
}

pub fn generate_doctor_schedules(
    meta: Vec<Doctor>,
    days: Vec<&'static str>,
    shifts: Vec<(NaiveTime, NaiveTime)>,
) -> Vec<doctor_schedules::ActiveModel> {
    let mut rng = rand::rng();

    let mut schedule = Vec::new();
    for doctor in meta {
        let num_schedule = rng.random_range(2..=3);
        let selected_days = days.choose_multiple(&mut rng, num_schedule);

        for day in selected_days {
            let (start, end) = shifts.choose(&mut rng).unwrap();
            schedule.push(doctor_schedules::ActiveModel {
                doctor_id: Set(doctor.id),
                polyclinic_id: Set(doctor.poly_id),
                room_code: Set(doctor.room_code.clone()),
                day_of_week: Set(day.to_string()),
                start_time: Set(*start),
                end_time: Set(*end),
                status: Set("ACTIVE".to_string()),
                ..Default::default()
            });
        }
    }

    schedule
}

pub fn generate_nurses_polyclinic_assignments(
    nurses_poly: Vec<(i32, i32)>,
) -> Vec<nurses_polyclinic_assignments::ActiveModel> {
    let mut rng = rand::rng();
    let now = Utc::now().naive_utc();
    let mut assignments = Vec::new();

    for (nurse_id, poly_id) in nurses_poly {
        let days_ago = rng.random_range(30..=365);
        let assigned_since = now - Duration::days(days_ago);

        let assigned_until = if rng.random_bool(0.3) {
            Some(assigned_since + Duration::days(rng.random_range(30..=180)))
        } else {
            None
        };

        let notes = if rng.random_bool(0.5) {
            Some("Temporary assignment".to_string())
        } else {
            None
        };

        assignments.push(nurses_polyclinic_assignments::ActiveModel {
            nurse_id: Set(nurse_id),
            polyclinic_id: Set(poly_id),
            assigned_since: Set(assigned_since),
            assigned_until: Set(assigned_until),
            notes: Set(notes),
            ..Default::default()
        });
    }

    assignments
}

pub fn generate_users(employee_ids_and_dept: Vec<(i32, String)>) -> Vec<user::ActiveModel> {
    let mut rng = rand::rng();
    let now = Utc::now();

    let dept_to_role = |dept: String| -> Role {
        match dept.as_str() {
            "DPT01" => Role::Admin,
            "DPT02" => Role::Staff,
            "DPT03" => Role::Cashier,
            "DPT04" => Role::Staff,
            "DPT05" => Role::Doctor,
            "DPT06" => Role::Emergency,
            "DPT07" => Role::Staff,
            "DPT08" => Role::Nurse,
            "DPT09" => Role::LabStaff,
            "DPT10" => Role::Staff,
            _ => Role::Staff,
        }
    };

    let mut users = Vec::new();
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(8, 1, 1, None).unwrap(),
    );
    for (id, dept) in employee_ids_and_dept {
        let role = dept_to_role(dept);
        let username = format!("user_{}", id);
        let password_raw = format!("{}{}", username, role.to_string().to_lowercase());

        let salt = SaltString::generate(&mut OsRng);

        let password_hash = argon2
            .hash_password(password_raw.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();

        users.push(user::ActiveModel {
            employee_id: Set(id),
            password: Set(password_hash),
            username: Set(username),
            role: Set(role),
            last_login: Set(Some(
                (now - Duration::days(rng.random_range(1..=30))).naive_utc(),
            )),
            is_active: Set(true),
            ..Default::default()
        });
    }

    users
}
