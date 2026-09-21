use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub name: String,
    pub generated_by: String,
    pub generated_at: DateTime<Utc>,
    pub content_json: String,
}
