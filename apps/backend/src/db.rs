use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use pqc_common::{
    rules::{evaluate_observation, PqcObservationSummary},
    types::Asset as AssetRecord,
};
use rusqlite::{params, Connection, Result as SqliteResult};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PruneSummary {
    pub observations_deleted: usize,
    pub cutoff: String,
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
                status TEXT NOT NULL,
                scope_state TEXT NOT NULL DEFAULT 'pending_review'
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

            CREATE TABLE IF NOT EXISTS sensor_tokens (
                id TEXT PRIMARY KEY,
                sensor_id TEXT NOT NULL,
                token_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                revoked_at TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                sensor_id TEXT NOT NULL,
                device_id TEXT,
                asset_id TEXT,
                observation_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                collected_at TEXT NOT NULL,
                evidence_confidence TEXT NOT NULL DEFAULT 'direct'
            );

            CREATE TABLE IF NOT EXISTS services (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                protocol TEXT NOT NULL,
                port INTEGER,
                service_name TEXT,
                version TEXT,
                status TEXT NOT NULL DEFAULT 'observed',
                first_seen_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
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
        let _ = conn.execute(
            "ALTER TABLE assets ADD COLUMN scope_state TEXT NOT NULL DEFAULT 'pending_review'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE observations ADD COLUMN evidence_confidence TEXT NOT NULL DEFAULT 'direct'",
            [],
        );
        Ok(())
    }

    pub fn connect(&self) -> SqliteResult<Connection> {
        Connection::open(self.url())
    }

    pub fn dashboard_counts(&self) -> SqliteResult<DashboardCounts> {
        let conn = self.connect()?;
        let assets = conn.query_row(
            "SELECT COUNT(*) FROM assets WHERE scope_state = 'included'",
            [],
            |row| row.get(0),
        )?;
        let sensors = conn.query_row(
            "SELECT COUNT(*) FROM sensors WHERE status = 'online'",
            [],
            |row| row.get(0),
        )?;
        let observations =
            conn.query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))?;
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

    pub fn prune_observations(&self, cutoff: &str, actor_id: &str) -> SqliteResult<PruneSummary> {
        let conn = self.connect()?;
        let deleted = conn.execute(
            "DELETE FROM observations WHERE collected_at < ?1",
            params![cutoff],
        )?;
        let details = serde_json::json!({
            "observations_deleted": deleted,
            "cutoff": cutoff,
        });
        conn.execute(
            r#"
            INSERT INTO audit_logs (
                id, actor_type, actor_id, action, target_type, created_at, details_json
            ) VALUES (?1, 'admin', ?2, 'telemetry_pruned', 'observations', ?3, ?4)
            "#,
            params![
                format!("audit-{}", Uuid::new_v4()),
                actor_id,
                chrono::Utc::now().to_rfc3339(),
                details.to_string(),
            ],
        )?;

        Ok(PruneSummary {
            observations_deleted: deleted,
            cutoff: cutoff.to_string(),
        })
    }
}

pub struct SensorRepository {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct SensorSummary {
    pub id: String,
    pub name: String,
    pub sensor_type: String,
    pub status: String,
    pub last_seen: Option<String>,
}

impl SensorRepository {
    pub fn new(db: &Database) -> SqliteResult<Self> {
        Ok(Self {
            conn: db.connect()?,
        })
    }

    pub fn register(&self, id: &str, name: &str, sensor_type: &str) -> SqliteResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO sensors (id, name, sensor_type, status, config_json)
            VALUES (?1, ?2, ?3, 'offline', '{}')
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                sensor_type = excluded.sensor_type
            "#,
            params![id, name, sensor_type],
        )?;
        Ok(())
    }

    pub fn heartbeat(&self, id: &str, seen_at: &str) -> SqliteResult<bool> {
        let updated = self.conn.execute(
            "UPDATE sensors SET status = 'online', last_seen = ?1 WHERE id = ?2",
            params![seen_at, id],
        )?;
        Ok(updated == 1)
    }

    pub fn exists(&self, id: &str) -> SqliteResult<bool> {
        self.conn
            .query_row("SELECT 1 FROM sensors WHERE id = ?1 LIMIT 1", params![id], |_| {
                Ok(())
            })
            .map(|_| true)
            .or_else(|error| {
                if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                    Ok(false)
                } else {
                    Err(error)
                }
            })
    }

    pub fn list(&self) -> SqliteResult<Vec<SensorSummary>> {
        let mut statement = self.conn.prepare(
            "SELECT id, name, sensor_type, status, last_seen FROM sensors ORDER BY name ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(SensorSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                sensor_type: row.get(2)?,
                status: row.get(3)?,
                last_seen: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}

pub struct SensorTokenRepository {
    conn: Connection,
}

impl SensorTokenRepository {
    pub fn new(db: &Database) -> SqliteResult<Self> {
        Ok(Self {
            conn: db.connect()?,
        })
    }

    pub fn issue(&self, sensor_id: &str, expires_at: &str) -> SqliteResult<String> {
        let token = Uuid::new_v4().to_string();
        let token_hash = hash_token(&token);
        self.conn.execute(
            "INSERT INTO sensor_tokens (id, sensor_id, token_hash, expires_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![Uuid::new_v4().to_string(), sensor_id, token_hash, expires_at, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(token)
    }

    pub fn is_valid(&self, sensor_id: &str, token: &str, now: &str) -> SqliteResult<bool> {
        let token_hash = hash_token(token);
        let mut statement = self.conn.prepare(
            "SELECT 1 FROM sensor_tokens WHERE sensor_id = ?1 AND token_hash = ?2 AND revoked_at IS NULL AND expires_at > ?3 LIMIT 1",
        )?;
        Ok(statement
            .query_row(params![sensor_id, token_hash, now], |_| Ok(()))
            .is_ok())
    }

    pub fn revoke_sensor(&self, sensor_id: &str, revoked_at: &str) -> SqliteResult<usize> {
        self.conn.execute(
            "UPDATE sensor_tokens SET revoked_at = ?1 WHERE sensor_id = ?2 AND revoked_at IS NULL",
            params![revoked_at, sensor_id],
        )
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
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

    pub fn set_scope(&self, asset_id: &str, scope_state: &str) -> SqliteResult<bool> {
        let changed = self.conn.execute(
            "UPDATE assets SET scope_state = ?1 WHERE id = ?2",
            params![scope_state, asset_id],
        )?;
        Ok(changed == 1)
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
                format!("{:?}", asset.status),
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
                status: match row.get::<_, String>(7)?.as_str() {
                    "Active" => pqc_common::types::AssetStatus::Active,
                    "Inactive" => pqc_common::types::AssetStatus::Inactive,
                    _ => pqc_common::types::AssetStatus::Unknown,
                },
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ObservationSummary {
    pub id: String,
    pub sensor_id: String,
    pub device_id: Option<String>,
    pub asset_id: Option<String>,
    pub observation_type: String,
    pub payload: serde_json::Value,
    pub collected_at: String,
    pub evidence_confidence: String,
}

impl ObservationRepository {
    pub fn new(db: &Database) -> SqliteResult<Self> {
        Ok(Self {
            conn: db.connect()?,
        })
    }

    pub fn recent(&self, limit: i64) -> SqliteResult<Vec<ObservationSummary>> {
        let bounded_limit = limit.clamp(1, 100);
        let mut statement = self.conn.prepare(
            "SELECT id, sensor_id, device_id, asset_id, observation_type, payload_json, collected_at, evidence_confidence FROM observations ORDER BY collected_at DESC LIMIT ?1",
        )?;
        let rows = statement.query_map(params![bounded_limit], |row| {
            let payload_json: String = row.get(5)?;
            Ok(ObservationSummary {
                id: row.get(0)?,
                sensor_id: row.get(1)?,
                device_id: row.get(2)?,
                asset_id: row.get(3)?,
                observation_type: row.get(4)?,
                payload: serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null),
                collected_at: row.get(6)?,
                evidence_confidence: row.get(7)?,
            })
        })?;
        rows.collect()
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
        let duplicate = self.conn.query_row(
            r#"
            SELECT 1 FROM observations
            WHERE sensor_id = ?1
              AND device_id IS ?2
              AND asset_id IS ?3
              AND observation_type = ?4
              AND payload_json = ?5
              AND collected_at = ?6
            LIMIT 1
            "#,
            params![
                sensor_id,
                device_id,
                asset_id,
                observation_type,
                payload_json,
                collected_at,
            ],
            |_| Ok(true),
        );
        if duplicate.unwrap_or(false) {
            return Ok(());
        }

        self.normalize_asset(asset_id, device_id, observation_type, payload_json, collected_at)?;
        let evidence_confidence = evidence_confidence(payload_json);
        self.normalize_service(asset_id.or(device_id), observation_type, payload_json, collected_at)?;

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
                collected_at,
                evidence_confidence
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                id,
                sensor_id,
                device_id,
                asset_id,
                observation_type,
                payload_json,
                collected_at,
                evidence_confidence,
            ],
        )?;

        let evaluation = parse_summary(observation_type, payload_json);
        let assessment = evaluate_observation(&evaluation);
        let findings_json =
            serde_json::to_string(&assessment.findings).unwrap_or_else(|_| "[]".to_string());

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

    fn normalize_asset(
        &self,
        asset_id: Option<&str>,
        device_id: Option<&str>,
        observation_type: &str,
        payload_json: &str,
        collected_at: &str,
    ) -> SqliteResult<()> {
        let id = asset_id.or(device_id).unwrap_or("unknown-asset");
        let payload: serde_json::Value = serde_json::from_str(payload_json).unwrap_or_default();
        let hostname = payload
            .get("hostname")
            .or_else(|| payload.get("host"))
            .and_then(|value| value.as_str());
        let mut ip_addresses = payload
            .get("ip_addresses")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for key in ["source_ip", "destination_ip", "ip"] {
            if let Some(ip) = payload.get(key).and_then(|value| value.as_str()) {
                if !ip_addresses.iter().any(|existing| existing == ip) {
                    ip_addresses.push(ip.to_string());
                }
            }
        }
        let ip_addresses = ip_addresses.join(",");
        let device_type = payload
            .get("device_type")
            .and_then(|value| value.as_str())
            .unwrap_or(observation_type);

        self.conn.execute(
            r#"
            INSERT INTO assets (id, hostname, ip_addresses, device_type, first_seen, last_seen, status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?5, 'Active')
            ON CONFLICT(id) DO UPDATE SET
                hostname = COALESCE(excluded.hostname, assets.hostname),
                ip_addresses = CASE
                    WHEN excluded.ip_addresses = '' THEN assets.ip_addresses
                    ELSE excluded.ip_addresses
                END,
                device_type = excluded.device_type,
                last_seen = excluded.last_seen,
                status = 'Active'
            "#,
            params![id, hostname, ip_addresses, device_type, collected_at],
        )?;
        Ok(())
    }

    fn normalize_service(
        &self,
        asset_id: Option<&str>,
        observation_type: &str,
        payload_json: &str,
        collected_at: &str,
    ) -> SqliteResult<()> {
        let Some(asset_id) = asset_id else {
            return Ok(());
        };
        let payload: serde_json::Value = serde_json::from_str(payload_json).unwrap_or_default();
        let protocol = payload
            .get("protocol")
            .and_then(|value| value.as_str())
            .unwrap_or(observation_type);
        let port = payload.get("port").and_then(|value| value.as_i64());
        let service_name = payload
            .get("service_name")
            .or_else(|| payload.get("application"))
            .and_then(|value| value.as_str());
        let version = payload
            .get("version")
            .or_else(|| payload.get("tls_version"))
            .or_else(|| payload.get("ssh_version"))
            .and_then(|value| value.as_str());
        let id = format!("{}:{}:{}", asset_id, protocol, port.unwrap_or(0));

        self.conn.execute(
            r#"
            INSERT INTO services (
                id, asset_id, protocol, port, service_name, version,
                first_seen_at, last_seen_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(id) DO UPDATE SET
                service_name = COALESCE(excluded.service_name, services.service_name),
                version = COALESCE(excluded.version, services.version),
                last_seen_at = excluded.last_seen_at,
                status = 'observed'
            "#,
            params![id, asset_id, protocol, port, service_name, version, collected_at],
        )?;
        Ok(())
    }
}

fn evidence_confidence(payload_json: &str) -> &'static str {
    let payload: serde_json::Value = serde_json::from_str(payload_json).unwrap_or_default();
    match payload
        .get("evidence_confidence")
        .and_then(|value| value.as_str())
    {
        Some("inferred") => "inferred",
        Some("insufficient") => "insufficient",
        _ => "direct",
    }
}

fn parse_summary(observation_type: &str, payload_json: &str) -> PqcObservationSummary {
    let parsed: serde_json::Value = serde_json::from_str(payload_json).unwrap_or_default();
    let protocol = parsed
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or(observation_type)
        .to_string();
    let tls_version = parsed
        .get("tls_version")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let ssh_version = parsed
        .get("ssh_version")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let key_type = parsed
        .get("key_type")
        .and_then(|v| v.as_str())
        .map(str::to_string);
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
            id: format!(
                "asset-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
            hostname: Some("web-01".to_string()),
            ip_addresses: vec!["10.0.0.10".to_string()],
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            device_type: "server".to_string(),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            status: pqc_common::types::AssetStatus::Active,
        };

        repo.save_asset(&asset).unwrap();
        let assets = repo.list_assets().unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, asset.id);
        assert_eq!(assets[0].hostname, Some("web-01".to_string()));
    }

    #[test]
    fn sensor_repository_tracks_registration_and_heartbeat() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let db = Database::new(&db_url);
        db.init().unwrap();

        let repository = SensorRepository::new(&db).unwrap();
        assert!(!repository.exists("sensor-1").unwrap());

        repository
            .register("sensor-1", "Network Sensor", "network")
            .unwrap();
        assert!(repository.exists("sensor-1").unwrap());
        assert!(repository
            .heartbeat("sensor-1", "2026-09-23T12:00:00Z")
            .unwrap());

        let status: String = db
            .connect()
            .unwrap()
            .query_row("SELECT status FROM sensors WHERE id = 'sensor-1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "online");
    }

    #[test]
    fn duplicate_observations_are_ignored() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let db = Database::new(&db_url);
        db.init().unwrap();
        let repository = ObservationRepository::new(&db).unwrap();

        for _ in 0..2 {
            repository
                .save_observation(
                    "sensor-1",
                    Some("device-1"),
                    Some("asset-1"),
                    "tls",
                    r#"{"tls_version":"1.3"}"#,
                    "2026-09-23T12:00:00Z",
                )
                .unwrap();
        }

        let observations: i64 = db
            .connect()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))
            .unwrap();
        let assessments: i64 = db
            .connect()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM assessments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(observations, 1);
        assert_eq!(assessments, 1);
    }

    #[test]
    fn observation_normalization_creates_asset() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let db = Database::new(&db_url);
        db.init().unwrap();
        let repository = ObservationRepository::new(&db).unwrap();

        repository
            .save_observation(
                "sensor-1",
                Some("device-1"),
                Some("asset-1"),
                "tls",
                r#"{"hostname":"web-01","ip_addresses":["10.0.0.10"],"device_type":"server","port":443,"service_name":"https","evidence_confidence":"inferred"}"#,
                "2026-09-23T12:00:00Z",
            )
            .unwrap();

        let asset: (String, String, String) = db
            .connect()
            .unwrap()
            .query_row(
                "SELECT hostname, ip_addresses, device_type FROM assets WHERE id = 'asset-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(asset, ("web-01".to_string(), "10.0.0.10".to_string(), "server".to_string()));

        let service: (String, i64, String, String) = db
            .connect()
            .unwrap()
            .query_row(
                "SELECT protocol, port, service_name, version FROM services WHERE asset_id = 'asset-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get::<_, Option<String>>(3)?.unwrap_or_default())),
            )
            .unwrap();
        assert_eq!(service, ("tls".to_string(), 443, "https".to_string(), String::new()));

        let confidence: String = db
            .connect()
            .unwrap()
            .query_row("SELECT evidence_confidence FROM observations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(confidence, "inferred");

        assert_eq!(db.dashboard_counts().unwrap().assets, 0);
        let assets = AssetRepository::new(&db).unwrap();
        assert!(assets.set_scope("asset-1", "included").unwrap());
        assert_eq!(db.dashboard_counts().unwrap().assets, 1);
    }

    #[test]
    fn pruning_removes_old_observations_and_audits_deletion() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let db = Database::new(&db_url);
        db.init().unwrap();
        let repository = ObservationRepository::new(&db).unwrap();

        repository
            .save_observation(
                "sensor-1",
                Some("device-1"),
                Some("asset-1"),
                "tls",
                r#"{"tls_version":"1.2"}"#,
                "2020-01-01T00:00:00Z",
            )
            .unwrap();
        repository
            .save_observation(
                "sensor-1",
                Some("device-1"),
                Some("asset-1"),
                "tls",
                r#"{"tls_version":"1.3"}"#,
                "2099-01-01T00:00:00Z",
            )
            .unwrap();

        let summary = db
            .prune_observations("2021-01-01T00:00:00Z", "platform-admin")
            .unwrap();
        assert_eq!(summary.observations_deleted, 1);

        let remaining: i64 = db
            .connect()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))
            .unwrap();
        let audit_events: i64 = db
            .connect()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM audit_logs WHERE action = 'telemetry_pruned'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 1);
        assert_eq!(audit_events, 1);
    }
}
