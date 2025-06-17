use chrono::{NaiveDate, Utc};
use regex::Regex;
use validator::ValidationError;

pub fn validate_phone_number(phone_number: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^(?:\+?\d{1,3}|0)\d{0,14}$").expect("Failed to parse regex");
    if re.is_match(phone_number) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_phone_number").with_message(std::borrow::Cow::Borrowed("Phone number must start with +xxx, +xx, +x, or 0 and be a maximum of 15 characters long.")))
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, ValidationError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ValidationError::new("invalid_date_format"))
}

pub fn validate_birth_date(birth_date: &str) -> Result<(), ValidationError> {
    let date = parse_date(birth_date)?;
    let today = Utc::now().naive_utc().date();

    if date > today {
        return Err(ValidationError::new("birth_date_future"));
    }

    let age = today
        .years_since(date)
        .expect("Failed to calculate delta year");

    if age < 17 {
        return Err(ValidationError::new("birth_date_too_young"));
    }

    Ok(())
}

pub fn validate_hire_date(hire_date: &str) -> Result<(), ValidationError> {
    let date = parse_date(hire_date)?;
    let today = Utc::now().naive_utc().date();

    if date > today {
        return Err(ValidationError::new("hire_date_future"));
    }

    Ok(())
}

pub fn validate_department_code(dept_code: &str) -> Result<(), ValidationError> {
    let dept_codes = vec![
        "DPT01", // Administrative
        "DPT02", // Human Resource
        "DPT03", // Finance
        "DPT04", // IT
        "DPT05", // Clinical
        "DPT06", // Emergency
        "DPT07", // Procurement
        "DPT08", // Nursing
        "DPT09", // Laboratory
        "DPT10", // Support
    ];

    if !dept_codes.contains(&dept_code) {
        return Err(ValidationError::new("invalid_dept_code"));
    }
    Ok(())
}

pub fn validate_status_employee(status: &str) -> Result<(), ValidationError> {
    let employee_status = vec![
        "Permanent",
        "Contract",
        "Internship",
        "Resigned",
        "Terminated",
    ];

    if !employee_status.contains(&status) {
        return Err(ValidationError::new("invalid_employee_status"));
    }

    Ok(())
}

pub fn validate_gender(gender: &str) -> Result<(), ValidationError> {
    let genders = vec!["Male", "Female"];

    if !genders.contains(&gender) {
        return Err(ValidationError::new("invalid_gender"));
    }

    Ok(())
}

pub fn validate_employee_id(employee_id: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^\d+$").expect("Failed to parse regex");

    if re.is_match(employee_id) {
        Ok(())
    } else {
        Err(
            ValidationError::new("invalid_employee_id").with_message(std::borrow::Cow::Borrowed(
                "Employee id must be an absolute integer.",
            )),
        )
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^(?=.*[A-Z])(?=.*\d)(?=.*[@!/_\-&])[A-Za-z\d@!-/&]{8,}$")
        .expect("Failed to define regex");

    if re.is_match(password) {
        Ok(())
    } else {
        Err(ValidationError::new("password_invalid").with_message(std::borrow::Cow::Borrowed(
            "Password must include Uppercase, number, special character, and at least 8 characters long.",
        )))
    }
}

pub fn validate_nip(nip: &str, birth_date: &str, gender: &str) -> Result<(), ValidationError> {
    // Check length
    if nip.len() != 18 || !nip.chars().all(|c| c.is_digit(10)) {
        return Err(ValidationError::new("invalid_nip_format")); // bukan 18 digit
    }

    // elaborate an NIP
    let birth_part = &nip[0..8];
    let gender_digit = &nip[14..15];
    let _serial = &nip[15..18];

    // Parse birth_date and match with NIP
    let birth = NaiveDate::parse_from_str(birth_date, "%Y-%m-%d")
        .map_err(|_| ValidationError::new("invalid_birth_date_format"))?;
    let birth_expected = birth.format("%Y%m%d").to_string();
    if birth_part != birth_expected {
        return Err(ValidationError::new("birth_date_mismatch"));
    }

    // Gender check
    match gender.to_lowercase().as_str() {
        "male" | "laki-laki" | "pria" => {
            if gender_digit != "1" {
                return Err(ValidationError::new("gender_digit_mismatch"));
            }
        }
        "female" | "perempuan" | "wanita" => {
            if gender_digit != "2" {
                return Err(ValidationError::new("gender_digit_mismatch"));
            }
        }
        _ => return Err(ValidationError::new("unknown_gender")),
    }

    Ok(())
}
