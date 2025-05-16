use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTriageResponse {
    pub patient_id: i32,
    pub visit_intent_id: i32,
    pub queue_number: i32,
    pub queue_type: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriageQueueResponse {
    pub visit_type: String,
    pub data: Vec<TriageQueueItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriageQueueItem {
    pub queue_number: i32,
    pub patient_id: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriageQueueStatus {
    pub queue_number: i32,
    pub queue_type: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriagePatientCalled {
    pub queue_number: i32,
    pub queue_type: String,
    pub called_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriageQueueComplete {
    pub queue_number: i32,
    pub queue_type: String,
    pub status: String,
    pub called_at: String,
    pub done_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriagePatientCancel {
    pub queue_number: i32,
    pub queue_type: String,
    pub previous_status: String,
    pub new_status: String,
}
