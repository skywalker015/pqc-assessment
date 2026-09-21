use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub hostname: Option<String>,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub device_type: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub status: AssetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub asset_id: String,
    pub port: u16,
    pub protocol: String,
    pub application: String,
    pub tls_detected: bool,
    pub ssh_detected: bool,
    pub fingerprint_confidence: f32,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigEvidence {
    pub device_id: String,
    pub openssl_version: Option<String>,
    pub openssh_version: Option<String>,
    pub user_public_keys: Vec<String>,
    pub ssh_config_path: Option<String>,
    pub tls_policy: Option<String>,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub id: String,
    pub asset_id: Option<String>,
    pub device_id: Option<String>,
    pub score: i32,
    pub status: AssessmentStatus,
    pub findings: Vec<Finding>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetStatus {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentStatus {
    Ready,
    AtRisk,
    NeedsAttention,
    Unknown,
}
