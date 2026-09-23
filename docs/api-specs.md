# PQC Readiness Platform API Specification

This document defines the primary REST API contracts between the frontend/sensors and the backend orchestrator. All endpoints are served over HTTPS. Sensor endpoints require scoped API-token authentication.

## 1. Sensor Telemetry Ingestion

### `POST /api/v1/sensors/telemetry`
**Description:** Accepts normalized telemetry from device, network, or remote sensors.
**Authentication:** Sensor API token in the `Authorization: Bearer <token>` header

**Request Body:**
```json
{
  "sensor_id": "network-01",
  "device_id": "dev-42",
  "observation_type": "tls_handshake",
  "observed_at": "2026-09-23T12:00:00Z",
  "payload": {
    "source_ip": "10.0.0.12",
    "destination_ip": "10.0.0.88",
    "port": 443,
    "tls_version": "1.3",
    "cipher": "TLS_AES_256_GCM_SHA384",
    "key_exchange": "X25519"
  }
}
```

**Response:** `202 Accepted`

## 2. Agent Management

### `GET /api/v1/agents/config`
**Description:** Allows an agent to poll for updated configurations (e.g., scan frequency, scopes).
**Authentication:** Sensor API token in the `Authorization: Bearer <token>` header

**Response:**
```json
{
  "interval_seconds": 3600,
  "scan_targets": ["10.0.0.0/24"],
  "rules_version": "v1.2.0"
}
```

## 3. Web Dashboard (Frontend)

### `GET /api/v1/assessments/summary`
**Description:** Returns the overall PQC readiness summary, asset posture counts, and mitigation progress for the dashboard.
**Authentication:** Bearer Token (JWT)

**Response:**
```json
{
  "overall_score": 65,
  "total_assets": 1200,
  "at_risk_assets": 450,
  "pqc_ready_assets": 750,
  "partially_ready_assets": 180,
  "not_pqc_ready_assets": 270,
  "unknown_assets": 90,
  "excluded_assets": 40,
  "mitigation": {
    "open_findings": 520,
    "in_progress_findings": 180,
    "mitigated_findings": 310,
    "newly_remediated_assets": 35,
    "regressed_assets": 4
  },
  "as_of": "2026-09-23T12:00:00Z"
}
```

The summary must be calculated only from assets with `scope_state` equal to `included`. Each asset detail response must expose its readiness state, scope state, evidence confidence, last evidence timestamp, failed rules, and mitigation status.

