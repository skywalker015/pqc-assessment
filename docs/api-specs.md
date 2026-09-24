# PQC Readiness Platform API Specification

This document defines the primary REST API contracts between the frontend/sensors and the backend orchestrator. The current sensor transport is TLS 1.3. WireGuard is deferred as an optional future deployment profile. Initial enrollment uses server-authenticated TLS without mTLS; subsequent certificate lifecycle and telemetry authorization are controlled by backend sensor state.

## 0. Sensor enrollment and certificate lifecycle

### `POST /api/v1/sensors/enrollment`
The sensor submits a CSR over the TLS 1.3 connection during initial enrollment.

Lifecycle rules:
- The sensor generates and retains its private key.
- The issuer CA signs only an approved CSR.
- End-entity certificates default to 30 days and are configurable.
- Renewal begins seven days before expiry.
- Failed renewal followed by expiry starts the new-sensor enrollment flow.
- Deleting a sensor marks it `deleted`; enrollment, renewal, and telemetry are denied.
- OCSP and CRL checks are not part of this design.

## 1. Sensor Telemetry Ingestion

### `POST /api/v1/sensors/telemetry`
**Description:** Accepts normalized telemetry from device, network, or remote sensors.
**Authentication:** Initial enrollment uses configured bootstrap controls over server-authenticated TLS 1.3. After enrollment, sensor certificate lifecycle state authorizes telemetry.

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
**Authentication:** TLS 1.3 with an active sensor certificate after enrollment.

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

