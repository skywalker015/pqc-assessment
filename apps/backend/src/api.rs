use axum::{
    routing::{get, post},
    response::Html,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{config::BackendConfig, db::Database};

const DASHBOARD_HTML: &str = include_str!("../../web/index.html");

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub sensor_id: String,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub accepted: bool,
    pub sensor_id: String,
    pub device_id: String,
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

async fn upload(Json(payload): Json<UploadRequest>) -> Json<UploadResponse> {
    Json(UploadResponse {
        accepted: true,
        sensor_id: payload.sensor_id,
        device_id: payload.device_id,
    })
}

async fn dashboard() -> Result<Json<DashboardResponse>, axum::http::StatusCode> {
    let database = Database::new(&BackendConfig::default().database_url);
    let counts = database
        .dashboard_counts()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let overall_score = if counts.observations == 0 {
        0
    } else {
        (100_i64 - (counts.high_risk * 20).min(100)) as u8
    };

    let mut recommended_actions = Vec::new();
    if counts.observations == 0 {
        recommended_actions.push("Run a sensor to collect the first environment observations".to_string());
    }
    if counts.high_risk > 0 {
        recommended_actions.push("Review high-risk findings and assign remediation owners".to_string());
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

pub fn router() -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/health", get(health))
        .route("/api/dashboard", get(dashboard))
        .route("/api/uploads", post(upload))
}
