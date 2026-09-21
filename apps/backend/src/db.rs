use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use pqc_common::{
    models::asset::AssetRecord,
    rules::{evaluate_observation, PqcObservationSummary},
};
use rusqlite::{params, Connection, Result as SqliteResult};

#[derive(Debug, Clone)]
pub struct Database {
    pub url: String,
}

fn normalize_sqlite_url(url: &str) -> String {
    if let Some(stripped) = url.strip_prefix("sqlite:") {
        stripped.to_string()
    } else {
        url.to_string()
    }
}

impl Database {
    pub fn new(url: &str) -> Self {
        let normalized = normalize_sqlite_url(url);
        let path = Path::new(&normalized);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        Self { url: normalized }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn init(&self) -> SqliteResult<()> {
        let conn = Connection::open(self.url())?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                hostname TEXT,
                ip_addresses TEXT NOT NULL,
                mac_address TEXT,
                device_type TEXT NOT NULL,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                status TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                asset_id TEXT,
                hostname TEXT NOT NULL,
                os_family TEXT,
                owner TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sensors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sensor_type TEXT NOT NULL,
                status TEXT NOT NULL,
                last_seen TEXT,
                config_json TEXT
            );

            CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                sensor_id TEXT NOT NULL,
                device_id TEXT,
                asset_id TEXT,
                observation_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                collected_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS assessments (
                id TEXT PRIMARY KEY,
                asset_id TEXT,
                device_id TEXT,
                score INTEGER NOT NULL,
                status TEXT NOT NULL,
                findings_json TEXT NOT NULL,
                generated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                actor_type TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                action TEXT NOT NULL,
                target_type TEXT,
                target_id TEXT,
                created_at TEXT NOT NULL,
                details_json TEXT
            );

            CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                generated_by TEXT NOT NULL,
                generated_at TEXT NOT NULL,
                content_json TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn connect(&self) -> SqliteResult<Connection> {
        Connection::open(self.url())
    }

    pub fn dashboard_counts(&self) -> SqliteResult<DashboardCounts> {
        let conn = self.connect()?;
        let assets = conn.query_row("SELECT COUNT(*) FROM assets", [], |row| row.get(0))?;
        let sensors = conn.query_row(
            "SELECT COUNT(*) FROM sensors WHERE status = 'online'",
            [],
            |row| row.get(0),
        )?;
        let observations = conn.query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))?;
        let high_risk = conn.query_row(
            "SELECT COUNT(*) FROM assessments WHERE status = 'high-risk'",
            [],
            |row| row.get(0),
        )?;

        Ok(DashboardCounts {
            assets,
            sensors,
            observations,
            high_risk,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DashboardCounts {
    pub assets: i64,
    pub sensors: i64,
    pub observations: i64,
    pub high_risk: i64,
}

pub struct AssetRepository {
    conn: Connection,
}

impl AssetRepository {
    pub fn new(db: &Database) -> SqliteResult<Self> {
        Ok(Self {
            conn: db.connect()?,
        })
    }

    pub fn save_asset(&self, asset: &AssetRecord) -> SqliteResult<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO assets (
                id,
                hostname,
                ip_addresses,
                mac_address,
                device_type,
                first_seen,
                last_seen,
                status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                asset.id,
                asset.hostname,
                asset.ip_addresses.join(","),
                asset.mac_address,
                asset.device_type,
                asset.first_seen.to_rfc3339(),
                asset.last_seen.to_rfc3339(),
                asset.status,
            ],
        )?;
        Ok(())
    }

    pub fn list_assets(&self) -> SqliteResult<Vec<AssetRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, hostname, ip_addresses, mac_address, device_type, first_seen, last_seen, status FROM assets",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AssetRecord {
                id: row.get(0)?,
                hostname: row.get(1)?,
                ip_addresses: row
                    .get::<_, String>(2)?
                    .split(',')
                    .filter(|v| !v.trim().is_empty())
                    .map(|v| v.to_string())
                    .collect(),
                mac_address: row.get(3)?,
                device_type: row.get(4)?,
                first_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                status: row.get(7)?,
            })
        })?;

        let mut assets = Vec::new();
        for asset in rows {
            assets.push(asset?);
        }
        Ok(assets)
    }
}

pub struct ObservationRepository {
    conn: Connection,
}

impl ObservationRepository {
    pub fn new(db: &Database) -> SqliteResult<Self> {
        Ok(Self {
            conn: db.connect()?,
        })
    }

    pub fn save_observation(
        &self,
        sensor_id: &str,
        device_id: Option<&str>,
        asset_id: Option<&str>,
        observation_type: &str,
        payload_json: &str,
        collected_at: &str,
    ) -> SqliteResult<()> {
        let id = format!(
            "obs-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        self.conn.execute(
            r#"
            INSERT INTO observations (
                id,
                sensor_id,
                device_id,
                asset_id,
                observation_type,
                payload_json,
                collected_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                id,
                sensor_id,
                device_id,
                asset_id,
                observation_type,
                payload_json,
                collected_at,
            ],
        )?;

        let evaluation = parse_summary(observation_type, payload_json);
        let assessment = evaluate_observation(&evaluation);
        let findings_json = serde_json::to_string(&assessment.findings).unwrap_or_else(|_| "[]".to_string());

        let assessment_id = format!(
            "assess-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        self.conn.execute(
            r#"
            INSERT INTO assessments (
                id,
                asset_id,
                device_id,
                score,
                status,
                findings_json,
                generated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                assessment_id,
                asset_id,
                device_id,
                assessment.score,
                assessment.status,
                findings_json,
                collected_at,
            ],
        )?;

        Ok(())
    }
}

fn parse_summary(observation_type: &str, payload_json: &str) -> PqcObservationSummary {
    let parsed: serde_json::Value = serde_json::from_str(payload_json).unwrap_or_default();
    let protocol = parsed.get("protocol").and_then(|v| v.as_str()).unwrap_or(observation_type).to_string();
    let tls_version = parsed.get("tls_version").and_then(|v| v.as_str()).map(str::to_string);
    let ssh_version = parsed.get("ssh_version").and_then(|v| v.as_str()).map(str::to_string);
    let key_type = parsed.get("key_type").and_then(|v| v.as_str()).map(str::to_string);
    let weak_crypto_detected = parsed
        .get("weak_crypto_detected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    PqcObservationSummary {
        protocol,
        tls_version,
        ssh_version,
        key_type,
        weak_crypto_detected,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::Utc;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn asset_repository_can_persist_and_list_assets() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let db = Database::new(&db_url);
        db.init().unwrap();

        let repo = AssetRepository::new(&db).unwrap();
        let asset = AssetRecord {
            id: format!("asset-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()),
            hostname: Some("web-01".to_string()),
            ip_addresses: vec!["10.0.0.10".to_string()],
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            device_type: "server".to_string(),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            status: "active".to_string(),
        };

        repo.save_asset(&asset).unwrap();
        let assets = repo.list_assets().unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, asset.id);
        assert_eq!(assets[0].hostname, Some("web-01".to_string()));
    }
}
