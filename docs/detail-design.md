# Detailed Design for the PQC Readiness Platform

## 1. Overview

This document defines the detailed software design for a modular PQC readiness assessment application. The system collects cryptography and configuration evidence from networks, devices, and applications, normalizes the data, produces readiness assessments, and presents a consolidated operational view through a web interface.

The design is intentionally componentized so each subsystem can evolve independently:
- web frontend
- backend orchestrator
- shared domain library
- network sensor
- device sensor
- remote sensor
- discovery sensor
- device inventory layer

The implementation target is Rust, with SQLite as the default database and PostgreSQL as the upgrade path.

---

## 2. Design goals

### Functional goals
- Detect encrypted traffic using TLS and SSH
- Discover assets on networks in a safe, non-intrusive way
- Inspect host crypto configuration
- Collect endpoint evidence for PQC readiness
- Summarize the posture of the environment
- Generate assessment and remediation reports
- Log all sensor and user activity

### Non-functional goals
- Clear modularity and separation of responsibility
- Easy local development and testing
- Safe handling of credentials and secrets
- Support for SQLite and PostgreSQL without major domain rewrites
- Resilience, observability, and auditability
- Low operational impact for passive and discovery sensors

---

## 3. System context

### Users
- Admin: manages environment, sensors, and reports
- Operator: runs scans and validates findings
- Auditor: reviews assessment and logs
- Target site owner: owns the devices being assessed

### External systems
- Managed devices
- Network segments and services
- SSH/TLS-enabled applications
- CSV uploads from local device agents
- Optional PostgreSQL database in enterprise deployments

### Primary use cases
1. Passive traffic discovery of TLS/SSH services
2. Asset detection via ping and lightweight discovery scan
3. Local device configuration collection
4. Remote device assessment using credentials
5. Central aggregation of results
6. Report generation and trend monitoring

---

## 4. Solution architecture

```text
+---------------------+      +----------------------+
| Web Frontend        | <-> | Backend API / Core   |
| apps/web            |      | apps/backend         |
+----------+----------+      +----------+-----------+
           |                                |
           |                                v
           |                     +--------------------+
           |                     | Shared Library     |
           |                     | libs/common        |
           |                     +--------------------+
           |                                |
           |        +-----------------------+-----------------------+
           |        |                       |                   |
           v        v                       v                   v
+----------------+  +----------------+  +----------------+  +----------------+
| Network Sensor |  | Device Agent   |  | Remote Agent   |  | Discovery      |
| sensors/network|  | sensors/device |  | sensors/remote |  | sensors/discovery|
+----------------+  +----------------+  +----------------+  +----------------+
        \                              |                     /
         \______________________________|______________________/
                                        |
                                        v
                            +---------------------+
                            | SQLite / Postgres   |
                            | Data Store          |
                            +---------------------+
```

---

## 5. Component design

## 5.1 Web frontend

Folder: `apps/web/`

### Responsibility
The frontend is the primary operational interface for administrators and operators. It provides visibility into the environment, configuration state, sensor activity, and assessment outcomes.

### Main pages
- Dashboard
- Assets
- Services and protocols
- Sensors
- Device inventory
- Upload CSV results
- Reports
- Audit logs
- Settings

### Functional capabilities
- Show overall readiness score and status
- Show counts and lists of PQC-ready, partially ready, not PQC-ready, and unknown assets
- Show mitigation progress, newly remediated assets, regressions, and stale evidence
- Display discovered assets and services
- List active sensors and last seen timestamps
- Upload CSV from supervised device agents
- Show assessment details with evidence trails
- Manage scan schedules and scope filters
- Provide user activity and audit information

### Technology direction
- Rust web frontend can be implemented as a server-rendered app or a JS-heavy SPA served from a Rust backend.

### Assessment classification contract

The backend must return these fields for each assessed asset:

- `scope_state`
- `readiness_state`
- `evidence_confidence`
- `last_evidence_at`
- `failed_rule_ids`
- `mitigation_status`
- `previous_readiness_state`

The backend must not collapse missing evidence into a passing or failing result. An asset with insufficient evidence is `unknown` until enough evidence is collected.
- For early phases, a simpler approach is a Rust backend serving HTML templates with progressive enhancement or a minimal JavaScript frontend.
- This keeps the stack manageable while preserving modularity.

### Key contract
- Consumes backend REST API
- Publishes upload endpoints for supervised agent files
- Reads assessment data and summary widgets

---

## 5.2 Backend service

Folder: `apps/backend/`

### Responsibility
The backend is the orchestrator and system of record. It accepts data from sensors, stores normalized results, runs assessment logic, and produces operational reports.

For real-time sensor telemetry, REST over HTTPS (HTTP/1.1) is used as the primary robust transport to ensure compatibility with corporate proxies and firewalls. Sensors authenticate with scoped API tokens or API keys sent in the `Authorization` header. Tokens must be stored securely, expire, rotate, and be revocable. gRPC is available as an optional secondary interface for low-latency network segments.

### Core modules
- API layer (REST primary, gRPC secondary)
- Agent Management & Configuration
- Data ingestion layer
- Repository layer
- Dynamic Assessment engine
- Audit/logging subsystem
- Data Retention & Pruning subsystem
- Scheduling subsystem
- File upload processor

### Runtime behavior
- Receives sensor submissions over HTTP or secure queueing
- Validates payload schemas
- Maps raw data to domain objects
- Persists to SQLite/PostgreSQL
- Triggers assessments
- Sends summary events to the web app

### API surface

#### Health and system
- `GET /health`
- `GET /api/version`

#### Assets
- `GET /api/assets`
- `GET /api/assets/:id`
- `POST /api/assets`
- `PATCH /api/assets/:id`

#### Sensors
- `GET /api/sensors`
- `POST /api/sensors/register`
- `PATCH /api/sensors/:id/status`

#### Agent ingestion
- `POST /api/agent/device/result`
- `POST /api/agent/discovery/result`
- `POST /api/agent/network/result`

#### Uploads
- `POST /api/uploads/csv`
- `GET /api/uploads/:id`

#### Assessments
- `GET /api/assessments`
- `GET /api/assessments/:id`
- `POST /api/assessments/run`

#### Reports
- `GET /api/reports`
- `POST /api/reports/generate`

#### Logs
- `GET /api/logs`
- `GET /api/logs/:id`

### Backend interface contract
Payloads should use JSON with a controlled schema. Example:

```json
{
  "sensor_id": "network-01",
  "device_id": "dev-42",
  "observed_at": "2026-09-20T12:00:00Z",
  "protocol": "tls",
  "source_ip": "10.0.0.12",
  "destination_ip": "10.0.0.88",
  "port": 443,
  "tls_version": "1.3",
  "cipher": "TLS_AES_256_GCM_SHA384",
  "status": "observed"
}
```

---

## 5.3 Shared library

Folder: `libs/common/`

### Responsibility
Contains the domain model and reusable logic used across all components.

### Modules
- `models/asset.rs`
- `models/device.rs`
- `models/service.rs`
- `models/observation.rs`
- `models/assessment.rs`
- `models/audit_log.rs`
- `db/mod.rs`
- `parser/csv.rs`
- `parser/json.rs`
- `rules/pqc_rules.rs`
- `reporting/mod.rs`
- `crypto/fingerprint.rs`

### Shared data model

#### Asset
```rust
struct Asset {
    id: String,
    hostname: Option<String>,
    ip_addresses: Vec<String>,
    mac_address: Option<String>,
    device_type: String,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    status: AssetStatus,
}
```

#### Service
```rust
struct Service {
    id: String,
    asset_id: String,
    port: u16,
    protocol: String,
    application: String,
    tls_detected: bool,
    ssh_detected: bool,
    fingerprint_confidence: f32,
    observed_at: DateTime<Utc>,
}
```

#### DeviceConfigEvidence
```rust
struct DeviceConfigEvidence {
    device_id: String,
    openssl_version: Option<String>,
    openssh_version: Option<String>,
    user_public_keys: Vec<String>,
    ssh_config_path: Option<String>,
    tls_policy: Option<String>,
    collected_at: DateTime<Utc>,
}
```

#### Assessment
```rust
struct Assessment {
    id: String,
    asset_id: Option<String>,
    device_id: Option<String>,
    score: i32,
    status: AssessmentStatus,
    findings: Vec<Finding>,
    generated_at: DateTime<Utc>,
}
```

### Configurable Rule Engine
Rules are implemented using an embedded dynamic scripting or policy engine (e.g., Rego/OPA, Rhai, or Lua) to allow updates to PQC readiness logic without recompiling the backend. Examples of dynamic rule evaluations:
- TLS 1.0 or 1.1 is still enabled
- OpenSSH version is unsupported
- Legacy SHA-1 certificates or weak RSA keys are present
- SSH public key algorithm is not PQC-ready or not policy-compliant
- Mixed key material or unapproved crypto policy is detected

---

## 5.4 Network sensor

Folder: `sensors/network/`

### Responsibility
The network sensor is passive. It inspects network flows and identifies TLS and SSH-related communication. It can also infer application identity, operating system hints, and probable host inventory.

### Core capabilities
- Capture packets or traffic metadata
- Recognize TLS handshakes and SSH banners
- Map flows to IPs and ports
- Infer service names and versions from banner metadata
- Build asset candidates from observed addresses
- Send normalized results to backend

### Passive data collection
- source IP and destination IP
- destination port
- TLS version
- handshake metadata
- SSH banner strings
- protocol classifications
- OS/application guesses with confidence

### Safety model
- Passive by default
- No aggressive or destructive traffic injection
- Only records metadata necessary for assessment

### Main interfaces
- Input: network packet stream or traffic capture metadata
- Output: `Observation` or `Service` records

---

## 5.5 Device sensor

Folder: `sensors/device/`

### Responsibility
This agent runs on the device and inspects local cryptographic configuration relevant to PQC readiness.

### Local evidence collected
- OpenSSL version
- OpenSSH version
- Installed crypto libraries and versions
- SSH configuration files
- User public keys in authorized_keys and user config
- TLS policy and supported versions
- Trust store state
- Potential weak or old certificate material

### Operational modes
#### Supervised mode
- Requires root or equivalent permissions
- Runs locally on the device
- Writes a CSV file
- User uploads file to the app

#### Unsupervised mode
- Runs on schedule or on demand
- Sends result JSON directly to backend
- Supports secure API auth and telemetry

### CSV format
The CSV schema should include:
- device_id
- hostname
- collection_time
- openssl_version
- openssh_version
- ssh_config_present
- user_public_key_count
- ssh_algo_summary
- tls_policy
- notes

Example:

```csv
device_id,hostname,collection_time,openssl_version,openssh_version,ssh_config_present,user_public_key_count,tls_policy
host-01,web-01,2026-09-20T12:00:00Z,OpenSSL 3.0.2,OpenSSH_9.6p1,yes,8,modern
```

### Security requirements
- Root execution is required only when local privileged inspection is needed
- Secrets should never be stored in logs
- CSV files should be validated before processing
- Result upload should be authenticated and rate-limited

---

## 5.6 Remote sensor

Folder: `sensors/remote/`

### Responsibility
This sensor remotely inspects devices using supplied credentials. It performs the same kinds of checks as the local device agent but through an authenticated remote session.

### Flow
1. Receive target host and credential information
2. Validate access
3. Connect via SSH or similar secure transport
4. Read configuration files and package versions
5. Normalize the findings into a common result format
6. Send results to backend

### Security rules
- Keep credential usage to the minimum required scope
- Prefer short-lived credentials or secure vaults
- Avoid writing secrets to logs
- Mask sensitive fields in UI rendering

### Example result payload
```json
{
  "sensor_id": "remote-agent-01",
  "device_id": "host-01",
  "target": "10.0.0.77",
  "collection_time": "2026-09-20T12:05:00Z",
  "openssl_version": "OpenSSL 3.0.2",
  "openssh_version": "OpenSSH_9.6p1",
  "public_key_count": 7,
  "status": "success"
}
```

---

## 5.7 Discovery sensor

Folder: `sensors/discovery/`

### Responsibility
This sensor discovers assets on a specified network using safe, non-intrusive discovery methods.

### Discovery methods
- Ping sweep
- Host enumeration
- Nmap host discovery in safe mode
- Lightweight banner checks

### Safety constraints
- No destructive scan patterns
- No dangerous NSE scripts or intrusive exploitation options
- Respect scan windows and access permissions
- Log full scan metadata

### Output
A list of likely hosts and their metadata, for example:
- IP address
- hostname
- port state summary
- service names
- discovery confidence

### Integration
- Results are stored as `AssetCandidate` entries and can be reviewed by operators
- The backend can queue deeper inspection for candidate devices

---

## 5.8 Device inventory module

Folder: `devices/`

### Responsibility
This subsystem models the managed devices and tracks which devices belong to which environments and sensors.

### Data fields
- id
- hostname
- asset_id
- network_segment
- os_family
- owner
- tags
- sensor_ids
- last_seen
- status

### Relationship model
- One device can have many services
- One device can have many sensor results
- One asset may map to multiple IPs or aliases
- Device inventory is a durable metadata model, while observation records are event-based

---

## 6. Data model and storage design

## 6.1 Database strategy

### SQLite default
- Local deployment
- Fast setup
- Good for teams and pilot environments
- Stored in `data/pqc_assessment.db`

### PostgreSQL upgrade path
- Configuration is abstracted behind repository interfaces
- Functions and queries are implemented in a database-specific module
- Schema is designed to support both engines

### Core tables

#### assets
- id
- hostname
- ip_addresses
- mac_address
- device_type
- first_seen
- last_seen
- status

#### devices
- id
- asset_id
- hostname
- os_family
- owner
- created_at
- updated_at

#### services
- id
- asset_id
- device_id
- port
- protocol
- application
- tls_detected
- ssh_detected
- fingerprint_confidence
- observed_at

#### observations
- id
- sensor_id
- device_id
- asset_id
- observation_type
- payload_json
- collected_at

#### agent_results
- id
- sensor_id
- device_id
- result_type
- status
- result_json
- collected_at

#### assessments
- id
- device_id
- asset_id
- score
- status
- findings_json
- created_at

#### reports
- id
- name
- generated_by
- generated_at
- content_json

#### audit_logs
- id
- actor_type
- actor_id
- action
- target_type
- target_id
- created_at
- details_json

#### users
- id
- username
- role
- created_at

---

## 7. Sequence flows

## 7.1 Passive network observation flow

```text
Network Sensor -> Backend API: observed traffic metadata
Backend -> Repository: store raw observation
Backend -> Assessment Engine: normalize and evaluate
Assessment Engine -> Reporting: generate environment findings
Backend -> Web Frontend: push summary and evidence
```

## 7.2 Supervised device agent upload flow

```text
Device Agent -> File System: write CSV result
Operator -> Web Frontend: upload CSV
Web Frontend -> Backend API: POST /api/uploads/csv
Backend -> Validation Layer: parse and validate schema
Backend -> Repository: save raw result and structured evidence
Backend -> Assessment Engine: evaluate
Backend -> Web UI: return assessment summary
```

## 7.3 Unsupervised device agent flow

```text
Device Agent -> Backend API: POST /api/agent/device/result
Backend -> Repository: persist evidence
Backend -> Assessment Engine: run readiness checks
Backend -> Web Frontend: update dashboard
```

## 7.4 Remote check flow

```text
Remote Sensor -> Remote Device: SSH / credentialed session
Remote Device -> Remote Sensor: configuration and version info
Remote Sensor -> Backend API: structured result
Backend -> Repository: save findings
Backend -> Assessment Engine: compute risk
```

---

## 8. Security design

### Compliance
- **OWASP ASVS:** The platform must comply with the OWASP Application Security Verification Standard (ASVS) at a minimum of Level 2. Level 3 controls must be applied where applicable, particularly for cryptographic asset management, session handling, and agent authentication.

### Identity and access
- All admin operations require authenticated sessions
- API endpoints should support role-based access control
- Device and sensor tokens should be scoped per environment or site

### Secret handling
- Credentials must not be stored in plain text
- Prefer secure vaults or encrypted environment variables
- Redact secrets in logs and reports

### Network protections
- Use PQC-ready encrypted transport (e.g., TLS 1.3 with ML-KEM/Kyber key exchange and ML-DSA/Dilithium certificates) for all backend communications
- Enforce scoped API-token or API-key authentication for all sensor telemetry uploads
- Implement secure enrollment, expiration, rotation, and revocation workflows for sensor tokens

### Auditability
Every action should be captured in `audit_logs` with enough detail to reconstruct the event.

---

## 9. Observability and operations

### Logging & Alerting
- Structured logs for API requests, sensor activity, and assessment runs
- Log levels: debug, info, warn, error
- Correlation ID for cross-component tracing
- **Alerting Strategy:** Alerts for critical events will be sent via email or displayed as dashboard alerts.

### Metrics
- Sensor success/failure rate
- Assets discovered
- Services identified
- Assessment scores over time
- Upload and parsing failures

### Health Checks & Incident Response
- **Agent Health Checks:** The backend will perform sensor/agent health checks every 30 minutes by default (customizable).
- **Incident Response for Failed Sensors:** If an agent fails its health check or disconnects, notifications will be sent via email and as a dashboard alert.
- Backend health endpoint and database connectivity checks.

### Data Retention & Pruning
- Aggregate repetitive observations (e.g., summarizing flow volume) rather than storing all raw network events.
- **Log Retention Rules:** A scheduled retention job purges telemetry and logs older than 90 days by default (this threshold is customizable).

### Backup and Recovery
- Backups can be scheduled via the application menu.
- Backups will be stored in a user-determined backup folder.
- All backups will be password-protected (encrypted), with the password set by the user.

### Concurrency Framework
- Standardize on the **Tokio** asynchronous runtime across the workspace to handle thousands of concurrent sensor API connections and I/O-heavy remote agent scanning without blocking threads

---

## 10. Testing strategy

### Unit tests
- parser validation for CSV and JSON
- rule engine behavior
- repository schema tests
- model serialization/deserialization

### Integration tests
- upload endpoint validation
- database persistence checks
- agent payload ingestion
- assessment generation from sensor results

### End-to-end tests
- workflow from discovery to report generation
- supervised CSV upload flow
- remote credentialed scan flow

---

## 11. Implementation phases

### Phase 1: Foundations
- Create Rust workspace and crate structure
- Configure SQLite repository
- Implement base models and validation
- Create backend skeleton and web shell

### Phase 2: Sensor core
- Implement network sensor passive traffic collection
- Add discovery sensor logic
- Create API integration for sensor telemetry

### Phase 3: Device and remote agents
- Implement local device inspection
- Add supervised CSV export path
- Implement remote credentialed checks

### Phase 4: Assessment and reporting
- Add rule catalog
- Generate score and findings
- Produce report exports

### Phase 5: Hardening
- Add RBAC and secret management
- Build audit logging and dashboards
- Prepare PostgreSQL migration support

---

## 12. Deployment model

### Local deployment
- SQLite database
- Backend and web UI on the same host
- Sensors connect via local secure channels

### Enterprise deployment
- PostgreSQL database
- Backend in a service environment
- Sensors distributed across segments
- Central reporting and audit storage

### Recommended deployment topology
```text
[Web UI]
   |
[Backend API]
   |
[SQLite or PostgreSQL]
   /|
  / |
 [Network Sensor] [Discovery Sensor] [Remote Sensor]
                     |
               [Device Agent]
```

---

## 13. Risks and constraints

### Risks
- Sensors may produce incomplete or noisy evidence
- Some endpoints may not permit remote execution
- Public key metadata can be sensitive and must be handled carefully
- Credential-based remote checks require strict operational controls

### Constraints
- Non-intrusive asset discovery should avoid harmful scan patterns
- Data collection should respect site authorization boundaries
- All assessment logic must be explainable and auditable

---

## 14. Recommended next step

The next implementation step should be to scaffold the Rust workspace and create the first back-end/data layer with the shared models and SQLite repository. Once that foundation is working, the network and device sensors can be implemented independently against the same common domain contracts.
