use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRecord {
    pub id: String,
    pub sensor_id: String,
    pub device_id: Option<String>,
    pub asset_id: Option<String>,
    pub observation_type: String,
    pub payload_json: String,
    pub collected_at: DateTime<Utc>,
}
