use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use strum_macros::Display;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Debug, Validate)]
pub struct CreateTriageRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[serde(deserialize_with = "naive_date_time_to_naive_date")]
    pub date_of_birth: NaiveDate,
    #[validate(length(equal = 16, message = "National ID must be 16 digits"))]
    pub national_id: String,
    #[validate(length(equal = 13, message = "BPJS number must be 13 digits"))]
    pub bpjs_number: Option<String>,
    pub gender: Gender,
    #[validate(length(min = 1, message = "Emergency contact name is required"))]
    pub emergency_contact_name: String,
    #[validate(custom(function = "validate_phone"))]
    pub emergency_contact_phone: String,
    #[validate(length(min = 1, message = "Emergency contact relationship is required"))]
    pub emergency_contact_relationship: String,
    pub blood_type: BloodType,
    pub known_allergies: Option<String>,

    pub visit_type: VisitType,
}

fn naive_date_time_to_naive_date<'de, D>(d: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    if phone_regex.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid emergency contact phone"))
    }
}

#[derive(Debug, Clone, Display)]
pub enum BloodType {
    APlus,
    AMinus,
    BPlus,
    BMinus,
    ABPlus,
    ABMinus,
    OPlus,
    OMinus,
}

impl<'de> Deserialize<'de> for BloodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        match s.as_str() {
            "a+" | "aplus" => Ok(BloodType::APlus),
            "a-" | "aminus" => Ok(BloodType::AMinus),
            "b+" | "bplus" => Ok(BloodType::BPlus),
            "b-" | "bminus" => Ok(BloodType::BMinus),
            "ab+" | "abplus" => Ok(BloodType::ABPlus),
            "ab-" | "abminus" => Ok(BloodType::ABMinus),
            "o+" | "oplus" => Ok(BloodType::OPlus),
            "o-" | "ominus" => Ok(BloodType::OMinus),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown blood type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum VisitType {
    COMMON,
    BPJS,
}

impl<'de> Deserialize<'de> for VisitType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "common" | "Common" | "COMMON" => Ok(VisitType::COMMON),
            "bpjs" | "BPJS" => Ok(VisitType::BPJS),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown visit type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum Gender {
    Male,
    Female,
}

impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "male" | "m" | "Male" | "MALE" => Ok(Gender::Male),
            "female" | "f" | "Female" | "FEMALE" => Ok(Gender::Female),
            _ => Err(serde::de::Error::custom(format!("Unknown gender: {}", s))),
        }
    }
}
