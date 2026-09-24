# PQC Assessment Data Model

## 1. Purpose

This document defines the canonical data structures for the PQC assessment platform. It covers the core entities in the shared domain model and the persistence assumptions required for SQLite and PostgreSQL compatibility.

The shared domain layer is intended to live in `libs/common` and be used by the backend, sensors, and any future reporting or web components.

---

## 2. Core design principles

- use explicit, normalized entities rather than free-form telemetry blobs
- keep raw observations separate from normalized domain records
- support both SQLite and PostgreSQL without schema-specific logic in business code
- ensure every important change is auditable
- avoid storing plaintext secrets in the main application data store

---

## 3. Entity relationship overview

```text
Asset
  ├── has many Service
  ├── has many Observation
  ├── has many AssessmentResult
  └── belongs to a DeviceProfile (optional)

Sensor
  ├── emits many Observation
  ├── belongs to a SensorType
  ├── has status and heartbeat metadata
  └── has certificate lifecycle metadata; an optional future WireGuard peer identity may be added

Observation
  ├── belongs to an Asset
  ├── belongs to a Sensor
  └── may be linked to a Service or Protocol

Service
  ├── belongs to Asset
  └── has protocol, port, and version metadata

AssessmentResult
  ├── belongs to Asset
  ├── references one or more RuleEvaluation records
  └── stores overall score and status

RuleEvaluation
  ├── belongs to a RuleDefinition
  ├── belongs to an AssessmentResult
  └── stores pass/fail/unknown and evidence

AuditLog
  └── records user and system actions across the platform
```

### Assessment vocabulary

The data model must distinguish technical posture from remediation workflow:

- `readiness_state`: `pqc_ready`, `partially_ready`, `not_pqc_ready`, or `unknown`
- `scope_state`: `included`, `excluded`, or `pending_review`
- `evidence_confidence`: `direct`, `inferred`, or `insufficient`
- `mitigation_status`: `open`, `in_progress`, `mitigated`, `accepted_risk`, or `unverifiable`

These values must be stored explicitly rather than inferred only from a numeric score.

---

## 4. Entity definitions

### 4.1 Asset

Represents a discovered system or endpoint in the environment.

```rust
struct Asset {
    id: String,
    hostname: Option<String>,
    ip_addresses: Vec<String>,
    mac_address: Option<String>,
    device_type: String,
    asset_group: Option<String>,
    first_seen_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
    scope_state: String,
    owner: Option<String>,
}
```

Responsibilities:
- central entity for the inventory system
- stable identifier for all related evidence
- primary binding point for scores and findings

---

### 4.2 Service

Represents a protocol endpoint or listening service attached to an asset.

```rust
struct Service {
    id: String,
    asset_id: String,
    protocol: String,
    port: Option<i32>,
    service_name: Option<String>,
    version: Option<String>,
    status: String,
    first_seen_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
}
```

Typical protocol values:
- tls
- ssh
- http
- https
- smtp
- custom

---

### 4.3 Observation

The raw or normalized fact emitted by a sensor.

```rust
struct Observation {
    id: String,
    sensor_id: String,
    asset_id: Option<String>,
    service_id: Option<String>,
    kind: String,
    protocol: Option<String>,
    observed_at: DateTime<Utc>,
    payload_json: JsonValue,
    source_ip: Option<String>,
    destination_ip: Option<String>,
    confidence: Option<f64>,
}
```

Notes:
- `payload_json` stores protocol-specific details such as TLS version, cipher name, or OpenSSH version
- this model supports both raw telemetry and normalized findings
- keeping `payload_json` separate from typed fields allows flexibility during early development

---

### 4.4 Sensor

Represents a collection agent or monitoring endpoint.

```rust
struct Sensor {
    id: String,
    name: String,
    sensor_type: String,
    status: String,
    last_seen_at: Option<DateTime<Utc>>,
    metadata_json: JsonValue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    lifecycle_state: String,
    wireguard_peer_id: Option<String>,
    certificate_serial: Option<String>,
    certificate_fingerprint: Option<String>,
    certificate_expires_at: Option<DateTime<Utc>>,
    certificate_renewal_window_days: i32,
}
```

Allowed sensor types:
- network
- device
- remote
- discovery

---

### 4.5 AssessmentResult

Represents the evaluation of a single asset or environment against rules.

```rust
struct AssessmentResult {
    id: String,
    asset_id: Option<String>,
    environment_scope: String,
    score: i32,
    status: String,
    summary: String,
    readiness_state: String,
    evidence_confidence: String,
    mitigation_status: String,
    previous_assessment_id: Option<String>,
    generated_at: DateTime<Utc>,
    rule_version: String,
}
```

Status values:
- pass
- warning
- fail
- unknown

---

### 4.6 RuleDefinition

Defines a rule used to evaluate readiness.

```rust
struct RuleDefinition {
    id: String,
    code: String,
    title: String,
    description: String,
    severity: String,
    version: String,
    enabled: bool,
    created_at: DateTime<Utc>,
}
```

Examples:
- PQC-TLS-001
- PQC-TLS-002
- PQC-SSH-001
- PQC-KEY-001

---

### 4.7 RuleEvaluation

Stores the actual result of evaluating a rule for an asset.

```rust
struct RuleEvaluation {
    id: String,
    assessment_id: String,
    rule_id: String,
    result: String,
    evidence_json: JsonValue,
    message: String,
    mitigation_status: String,
    remediated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}
```

Result values:
- pass
- fail
- warning
- not_applicable

---

### 4.8 AuditLog

Captures important operational or policy events.

```rust
struct AuditLog {
    id: String,
    actor_type: String,
    actor_id: Option<String>,
    event_type: String,
    target_type: Option<String>,
    target_id: Option<String>,
    summary: String,
    metadata_json: JsonValue,
    created_at: DateTime<Utc>,
}
```

Examples:
- sensor registered
- assessment run created
- rule definition updated
- credential access recorded
- user changed scan scope

---

## 5. Database storage recommendations

### SQLite (default/local)
Use SQLite for local pilot deployments and single-host environments.

Recommended tables:
- assets
- services
- observations
- sensors
- assessment_results
- rule_definitions
- rule_evaluations
- audit_logs

Practical guidance:
- enable WAL mode for better concurrency
- add timestamps for all tables
- keep indices on asset_id, sensor_id, observed_at, and status fields

### PostgreSQL (enterprise)
Use PostgreSQL for larger or multi-sensor deployments.

Recommended additions:
- partitioning or retention buckets for observations by date
- JSONB fields where flexible telemetry payloads are common
- stronger operational constraints and explicit indexes

---

## 6. Indexing strategy

The following indexes are strongly recommended:

- `assets.last_seen_at`
- `services.asset_id`
- `observations.sensor_id`
- `observations.asset_id`
- `observations.observed_at`
- `assessment_results.asset_id`
- `assessment_results.generated_at`
- `audit_logs.created_at`
- `rule_evaluations.assessment_id`

---

## 7. Retention policy

Large telemetry volumes can rapidly grow the database. The platform should implement a clear retention policy such as:

- **Log Retention Rules:** keep raw observations for 90 days by default (this window is customizable by the user)
- keep aggregated summaries for longer durations
- retain audit logs according to compliance and internal policy
- prune stale sensor states and disconnected asset observations regularly

This policy should be implemented in a data retention job, not ad hoc cleanup scripts.

---

## 8. Data lifecycle rules

1. Sensors submit telemetry.
2. Backend validates and ingests payloads.
3. Data is mapped into `Observation` records.
4. The normalizer derives `Asset` and `Service` records.
5. Assessment engine runs against the latest evidence.
6. `AssessmentResult` and `RuleEvaluation` records persist the outcome.
7. Audit log records the action and provenance.
8. Retention job prunes stale raw observation data while preserving summary data.

---

## 9. Security constraints

- never persist plaintext credentials in `Observation`, `Asset`, or `AuditLog`
- encrypt secret-bearing metadata at rest or use a secret store
- do not treat `payload_json` as a place to store secrets or private keys
- ensure audit metadata is append-only and unambiguous

---

## 10. Example normalized record

```json
{
  "asset_id": "asset-001",
  "hostname": "web-01.internal",
  "ip_addresses": ["10.0.0.12"],
  "device_type": "server",
  "service": {
    "protocol": "tls",
    "port": 443,
    "service_name": "https",
    "version": "1.3",
    "status": "observed"
  },
  "observation": {
    "kind": "tls_handshake",
    "tls_version": "1.3",
    "cipher": "TLS_AES_256_GCM_SHA384",
    "key_exchange": "X25519"
  }
}
```

This is the kind of data that should be normalized into typed records and then evaluated by the rule engine.
