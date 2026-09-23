use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a discovered system or endpoint in the environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub hostname: Option<String>,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub device_type: String,
    pub asset_group: Option<String>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}
