use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTriageResponse {
    pub patient_id: i32,
    pub visit_intent_id: i32,
    pub queue_number: i32,
    pub queue_type: String,
    pub status: String,
}
