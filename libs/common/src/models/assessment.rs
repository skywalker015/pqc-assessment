use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRecord {
    pub id: String,
    pub asset_id: Option<String>,
    pub device_id: Option<String>,
    pub score: i32,
    pub status: String,
    pub findings: Vec<String>,
    pub generated_at: DateTime<Utc>,
}
