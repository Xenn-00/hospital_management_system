use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::infra::api::PaginationMeta;

#[derive(Debug, Serialize, Deserialize)]
pub struct PolyclinicSchedulesResponse {
    pub polyclinic_code: String,
    pub polyclinic_name: String,
    pub room_code: String,
    pub doctor_responsible: Vec<String>,
    pub schedules: Vec<PolySchedules>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolySchedules {
    pub day_of_week: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagedPolyclinicSchedulesResponse {
    pub data: Vec<PolyclinicSchedulesResponse>,
    pub meta: PaginationMeta,
}
