use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::Html,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    config::BackendConfig,
    db::{AssetRepository, Database, ObservationRepository, SensorRepository, SensorTokenRepository},
};

const DASHBOARD_HTML: &str = include_str!("../../web/index.html");
const MAX_UPLOAD_BODY_BYTES: usize = 1024 * 1024;
const MAX_IDENTIFIER_LENGTH: usize = 128;

#[derive(Clone)]
struct AppState {
    database: Database,
    sensor_api_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadRequest {
    pub sensor_id: String,
    pub device_id: String,
    #[serde(default = "default_observation_type")]
    pub observation_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

fn default_observation_type() -> String {
    "sensor-upload".to_string()
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub accepted: bool,
    pub sensor_id: String,
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SensorTokenRequest {
    pub sensor_id: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SensorTokenResponse {
    pub sensor_id: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct RevokeTokenResponse {
    pub sensor_id: String,
    pub revoked: usize,
}

#[derive(Debug, Deserialize)]
pub struct SensorRegistrationRequest {
    pub sensor_id: String,
    pub name: String,
    pub sensor_type: String,
}

#[derive(Debug, Serialize)]
pub struct SensorRegistrationResponse {
    pub registered: bool,
    pub sensor_id: String,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SensorListItem {
    pub sensor_id: String,
    pub name: String,
    pub sensor_type: String,
    pub status: String,
    pub last_seen: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeUpdateRequest {
    pub scope_state: String,
}

#[derive(Debug, Serialize)]
pub struct ScopeUpdateResponse {
    pub asset_id: String,
    pub scope_state: String,
}

#[derive(Debug, Deserialize)]
pub struct PruneRequest {
    pub retention_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub overall_score: u8,
    pub assets_detected: i64,
    pub sensors_online: i64,
    pub observations_collected: i64,
    pub high_risk_count: i64,
    pub recommended_actions: Vec<String>,
    pub sensor_status: Vec<String>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "pqc-backend",
    })
}

async fn upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UploadRequest>,
) -> Result<Json<UploadResponse>, axum::http::StatusCode> {
    let authorization = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let bootstrap_matches = state.sensor_api_token.as_deref().is_some_and(|expected| {
        let matches: bool = expected.as_bytes().ct_eq(authorization.as_bytes()).into();
        matches
    });
    let persisted_matches = if bootstrap_matches {
        false
    } else {
        let repository = SensorTokenRepository::new(&state.database)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        repository
            .is_valid(&payload.sensor_id, authorization, &Utc::now().to_rfc3339())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    if !bootstrap_matches && !persisted_matches {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !valid_identifier(&payload.sensor_id)
        || !valid_identifier(&payload.device_id)
        || !valid_identifier(&payload.observation_type)
        || payload
            .asset_id
            .as_deref()
            .is_some_and(|asset_id| !valid_identifier(asset_id))
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let sensor_repository = SensorRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !sensor_repository
        .heartbeat(&payload.sensor_id, &Utc::now().to_rfc3339())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let payload_json = serde_json::to_string(&payload.payload)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    if payload_json.len() > MAX_UPLOAD_BODY_BYTES {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let repository = ObservationRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    repository
        .save_observation(
            &payload.sensor_id,
            Some(&payload.device_id),
            payload.asset_id.as_deref(),
            &payload.observation_type,
            &payload_json,
            &Utc::now().to_rfc3339(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UploadResponse {
        accepted: true,
        sensor_id: payload.sensor_id,
        device_id: payload.device_id,
    }))
}

async fn issue_sensor_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SensorTokenRequest>,
) -> Result<Json<SensorTokenResponse>, StatusCode> {
    require_admin(&state, &headers)?;
    if payload.sensor_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let sensor_repository = SensorRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let registered = sensor_repository
        .exists(&payload.sensor_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !registered {
        return Err(StatusCode::NOT_FOUND);
    }

    let expires_at = payload
        .expires_at
        .unwrap_or_else(|| (Utc::now() + Duration::days(90)).to_rfc3339());
    let repository = SensorTokenRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = repository
        .issue(&payload.sensor_id, &expires_at)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SensorTokenResponse {
        sensor_id: payload.sensor_id,
        token,
        expires_at,
    }))
}

async fn register_sensor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SensorRegistrationRequest>,
) -> Result<Json<SensorRegistrationResponse>, StatusCode> {
    require_admin(&state, &headers)?;
    if payload.sensor_id.trim().is_empty()
        || payload.name.trim().is_empty()
        || payload.sensor_type.trim().is_empty()
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let sensor_repository = SensorRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sensor_repository
        .register(&payload.sensor_id, &payload.name, &payload.sensor_type)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SensorRegistrationResponse {
        registered: true,
        sensor_id: payload.sensor_id,
        status: "offline",
    }))
}

async fn list_sensors(
    State(state): State<AppState>,
) -> Result<Json<Vec<SensorListItem>>, StatusCode> {
    let repository = SensorRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sensors = repository
        .list()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|sensor| SensorListItem {
            sensor_id: sensor.id,
            name: sensor.name,
            sensor_type: sensor.sensor_type,
            status: sensor.status,
            last_seen: sensor.last_seen,
        })
        .collect();

    Ok(Json(sensors))
}

async fn telemetry_history(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::db::ObservationSummary>>, StatusCode> {
    let repository = ObservationRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let observations = repository
        .recent(25)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(observations))
}

async fn update_asset_scope(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(asset_id): Path<String>,
    Json(payload): Json<ScopeUpdateRequest>,
) -> Result<Json<ScopeUpdateResponse>, StatusCode> {
    require_admin(&state, &headers)?;
    if !valid_identifier(&asset_id)
        || !matches!(payload.scope_state.as_str(), "included" | "excluded" | "pending_review")
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repository = AssetRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !repository
        .set_scope(&asset_id, &payload.scope_state)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(ScopeUpdateResponse {
        asset_id,
        scope_state: payload.scope_state,
    }))
}

async fn prune_telemetry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PruneRequest>,
) -> Result<Json<crate::db::PruneSummary>, StatusCode> {
    require_admin(&state, &headers)?;
    let retention_days = payload.retention_days.unwrap_or(180).clamp(1, 3650);
    let cutoff = (Utc::now() - Duration::days(retention_days)).to_rfc3339();
    let summary = state
        .database
        .prune_observations(&cutoff, "platform-admin")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(summary))
}

async fn revoke_sensor_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sensor_id): Path<String>,
) -> Result<Json<RevokeTokenResponse>, StatusCode> {
    require_admin(&state, &headers)?;
    if sensor_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let repository = SensorTokenRepository::new(&state.database)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let revoked = repository
        .revoke_sensor(&sensor_id, &Utc::now().to_rfc3339())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RevokeTokenResponse { sensor_id, revoked }))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let expected = state
        .sensor_api_token
        .as_deref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let provided = bearer_token(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let matches: bool = expected.as_bytes().ct_eq(provided.as_bytes()).into();
    if matches {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn valid_identifier(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.len() <= MAX_IDENTIFIER_LENGTH
}

async fn dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardResponse>, axum::http::StatusCode> {
    let counts = state
        .database
        .dashboard_counts()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let overall_score = if counts.observations == 0 {
        0
    } else {
        (100_i64 - (counts.high_risk * 20).min(100)) as u8
    };

    let mut recommended_actions = Vec::new();
    if counts.observations == 0 {
        recommended_actions
            .push("Run a sensor to collect the first environment observations".to_string());
    }
    if counts.high_risk > 0 {
        recommended_actions
            .push("Review high-risk findings and assign remediation owners".to_string());
    }
    if counts.sensors == 0 {
        recommended_actions.push("Register and activate at least one sensor".to_string());
    }

    Ok(Json(DashboardResponse {
        overall_score,
        assets_detected: counts.assets,
        sensors_online: counts.sensors,
        observations_collected: counts.observations,
        high_risk_count: counts.high_risk,
        recommended_actions,
        sensor_status: vec![if counts.sensors == 0 {
            "No sensors online".to_string()
        } else {
            format!("{} sensor(s) online", counts.sensors)
        }],
    }))
}

async fn dashboard_page() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

fn router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/health", get(health))
        .route("/api/dashboard", get(dashboard))
        .route("/api/telemetry/history", get(telemetry_history))
        .route("/api/maintenance/prune", post(prune_telemetry))
        .route("/api/sensors", get(list_sensors).post(register_sensor))
        .route("/api/assets/:asset_id/scope", patch(update_asset_scope))
        .route("/api/uploads", post(upload))
        .route("/api/sensor-tokens", post(issue_sensor_token))
        .route(
            "/api/sensor-tokens/:sensor_id",
            axum::routing::delete(revoke_sensor_tokens),
        )
        .layer(RequestBodyLimitLayer::new(MAX_UPLOAD_BODY_BYTES))
        .with_state(state)
}

pub fn router() -> Router {
    let config = BackendConfig::default();
    let database = Database::new(&config.database_url);
    database.init().expect("failed to initialize database");

    router_with_state(AppState {
        database,
        sensor_api_token: config.sensor_api_token,
    })
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, body::Body, http::Request};
    use tempfile::NamedTempFile;
    use tower::ServiceExt;

    use super::*;

    async fn send_json(
        app: Router,
        method: &str,
        uri: &str,
        token: Option<&str>,
        payload: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let response = app
            .oneshot(request.body(Body::from(payload.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        (status, payload)
    }

    #[tokio::test]
    async fn sensor_token_lifecycle_controls_uploads() {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Database::new(&format!("sqlite:{}", temp_file.path().display()));
        database.init().unwrap();
        let app = router_with_state(AppState {
            database,
            sensor_api_token: Some("sensor-token".to_string()),
        });

        let (status, _) = send_json(
            app.clone(),
            "POST",
            "/api/sensors",
            Some("sensor-token"),
            serde_json::json!({
                "sensor_id": "sensor-1",
                "name": "Network Sensor",
                "sensor_type": "network"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, token_response) = send_json(
            app.clone(),
            "POST",
            "/api/sensor-tokens",
            Some("sensor-token"),
            serde_json::json!({"sensor_id": "sensor-1"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let sensor_token = token_response["token"].as_str().unwrap();

        let (status, _) = send_json(
            app.clone(),
            "POST",
            "/api/uploads",
            Some(sensor_token),
            serde_json::json!({
                "sensor_id": "sensor-1",
                "device_id": "device-1",
                "observation_type": "tls",
                "payload": {"tls_version": "1.2"}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = send_json(
            app.clone(),
            "DELETE",
            "/api/sensor-tokens/sensor-1",
            Some("sensor-token"),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let (status, _) = send_json(
            app,
            "POST",
            "/api/uploads",
            Some(sensor_token),
            serde_json::json!({
                "sensor_id": "sensor-1",
                "device_id": "device-1"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upload_rejects_oversized_identifiers() {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Database::new(&format!("sqlite:{}", temp_file.path().display()));
        database.init().unwrap();
        let app = router_with_state(AppState {
            database,
            sensor_api_token: Some("sensor-token".to_string()),
        });

        let (status, _) = send_json(
            app,
            "POST",
            "/api/uploads",
            Some("sensor-token"),
            serde_json::json!({
                "sensor_id": "s".repeat(MAX_IDENTIFIER_LENGTH + 1),
                "device_id": "device-1",
                "payload": {}
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
