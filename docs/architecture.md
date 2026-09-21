# PQC Readiness Application Design

## 1. Goal

Build a modular environment assessment platform that measures how ready an organization is for post-quantum cryptography (PQC) across networks, endpoints, and applications. The system collects evidence from distributed sensors, normalizes it, stores it in a database, performs assessments, and presents the results via a web interface.

The platform is designed to support:
- Passive network observation
- Local device-level configuration checks
- Remote credentialed checks
- Non-intrusive asset discovery
- SQLite for local/default use and PostgreSQL for enterprise upgrade
- Central orchestration and reporting

---

## 2. High-level architecture

The application is organized as a modular system with separate folders for each subsystem:

```text
pqc-assessment/
├── apps/
│   ├── web/
│   └── backend/
├── libs/
│   └── common/
├── sensors/
│   ├── network/
│   ├── device/
│   ├── remote/
│   └── discovery/
├── devices/
├── docs/
├── data/
├── scripts/
├── Cargo.toml
├── README.md
└── .gitignore
```

### Core components

1. Web frontend
   - Admin dashboard
   - Asset inventory
   - Sensor registration and status
   - Upload CSV results from supervised device agents
   - Assessment reports and trends

2. Backend service
   - Orchestrates agents
   - Receives sensor data
   - Stores and normalizes results
   - Runs assessment rules
   - Produces reports and audit logs

3. Shared library
   - Common domain models
   - Validation and parsing helpers
   - DB abstraction layer
   - Rule engine for PQC readiness
   - Report generation primitives

4. Sensor components
   - Network sensor: passive TLS/SSH traffic monitoring and asset discovery
   - Device agent: local endpoint inspection and file export
   - Remote agent: credentialed remote checks
   - Discovery agent: ping/nmap-based network asset discovery

5. Device components
   - Endpoint hosts, managed workstations, servers, appliances
   - Local agent deployment model
   - Secure credentials and runtime policies

---

## 3. Major responsibilities

### 3.1 Web frontend

Responsibilities:
- Show environment overview and readiness score
- Display inventory of assets and services
- Show sensor health and activity logs
- Allow admin to configure scan schedules and policies
- Support CSV upload for supervised device agent results
- Display audit history and security reports

Suggested pages:
- Dashboard
- Assets
- Sensors
- Device agents
- Assessments
- Reports
- Logs
- Settings

### 3.2 Backend service

Responsibilities:
- Receive telemetry from unsupervised agents
- Store raw observations and normalized records
- Schedule and coordinate scans
- Aggregate results from all sensors
- Run assessment logic against best-practice PQC rules
- Produce summary and risk reports
- Record user actions and agent activity in logs

The backend acts as the orchestrator and single source of truth for the application. For agent-to-backend communication, gRPC is the preferred protocol for structured telemetry, while the web UI can still consume JSON REST endpoints and upload files over HTTP.

### 3.3 Shared library

Responsibilities:
- Shared data structures:
  - Asset
  - Service
  - DeviceInfo
  - CertificateInfo
  - CryptoPolicy
  - Observation
  - AgentResult
  - Assessment
- Parsing of CSV and JSON payloads
- DB repository interfaces
- Rule engine for PQC readiness checks
- Report serialization helpers
- Common crypto fingerprinting logic

This library is where the domain logic lives so that each component can remain focused and easy to modify.

---

## 4. Sensor architecture

Each sensor has its own folder and specific purpose.

### 4.1 Network sensor

Folder: `sensors/network/`

Purpose:
- Passive monitoring of network traffic
- Detect TLS, SSH, SMTP with TLS, RDP with TLS, and other encrypted application flows
- Infer asset inventory from observed IP addresses and traffic patterns
- Estimate OS or application type using behavior and signatures

Functions:
- Packet capture / traffic inspection
- TLS handshake analysis
- SSH banner detection
- Service port identification
- Passive OS/application fingerprinting
- Asset discovery from observed IP ranges

Data collected:
- Source/destination IP
- Port numbers
- TLS version and cipher metadata
- SSH banner strings
- Protocols used
- Inferred service names
- Confidence score for OS/application guesses

Note:
- This is a passive sensor and should be non-intrusive by default.
- It should not initiate active probing beyond allowed observation on the monitored network path.

### 4.2 Device agent

Folder: `sensors/device/`

Purpose:
- Run on a device to inspect local PQC-related configuration
- Provide a supervised mode and an unsupervised mode

Local checks:
- OpenSSL version
- OpenSSH version
- User public keys
- SSH daemon configuration
- TLS library configuration
- Certificate bundle state
- Crypto policy settings
- System trust store status
- Potentially weak key algorithms

Operational modes:
1. Supervised mode
   - Requires root or equivalent permissions
   - Runs locally on the device
   - Writes results to CSV file
   - User uploads file to the web app

2. Unsupervised mode
   - Runs without local manual intervention
   - Pushes results to backend directly via API
   - Supports scheduled or event-driven execution

Output format:
- CSV for supervised mode
- JSON or secure transport payload for unsupervised mode

### 4.3 Remote agent

Folder: `sensors/remote/`

Purpose:
- Access target devices remotely using provided credentials
- Run the same local checks as the device agent but over the network
- Act as the unsupervised mode of the device agent for remote-managed systems

Functions:
- SSH login with user-supplied credentials
- Remote config collection
- Remote version detection
- Remote certificate and key inspection
- Credential validation and safe connection handling

Security constraints:
- Use least privilege where possible
- Do not store plaintext credentials long-term
- Use encrypted secret storage or temporary secure tokens
- Log credential usage without exposing secret material

### 4.4 Discovery agent

Folder: `sensors/discovery/`

Purpose:
- Discover assets in a specified network range
- Use non-intrusive scanning
- Identify devices that may need deeper inspection

Methods:
- ICMP ping sweep
- Non-intrusive service discovery using Nmap in safe configuration
- Passive validation of service banners
- Lightweight host enumeration

Safety rules:
- Use host discovery without destructive options
- Avoid intrusive port scanning beyond allowed policy
- Log all scan parameters to audit trail
- Respect rate limiting and authorization boundaries

---

## 5. Device model

Folder: `devices/`

Responsibilities:
- Represent the actual systems being assessed
- Distinguish between hosts, servers, network appliances, and endpoints
- Link assets to sensor results and observations

Example device types:
- Workstation
- Server
- Firewall
- Load balancer
- SSH gateway
- Mail relay
- VPN appliance

Each device stores:
- Device ID
- Hostname
- IPs and network segments
- Owner / admin context
- Operating system details
- Sensor assignments
- Last seen timestamp

---

## 6. Data flow

### Flow A: Passive network observation
1. Network sensor captures traffic and metadata
2. It identifies TLS/SSH-related traffic and service patterns
3. It infers assets and likely applications
4. Results are sent to backend
5. Backend stores raw events and normalized services

### Flow B: Device agent with supervision
1. Device agent runs locally with root privileges if required
2. It collects configuration and key material metadata
3. It writes CSV to disk
4. User uploads CSV via web frontend
5. Backend validates CSV and stores it
6. Assessment engine compares findings to PQC readiness rules

### Flow C: Device agent unsupervised
1. Device agent runs on schedule or on demand
2. It pushes JSON payload to backend API
3. Backend validates and stores data
4. Backend enqueues assessment tasks

### Flow D: Remote credentialed inspection
1. Remote agent receives host and credentials
2. It authenticates to target through SSH or similar interface
3. It collects crypto and config evidence
4. It sends results to backend

### Flow E: Discovery scan
1. Discovery agent enumerates network hosts with ping/Nmap
2. It identifies probable candidates for deeper inspection
3. Backend tags them as discovered assets and queues follow-up checks

---

## 7. Database design

### Default database: SQLite

Use SQLite for local deployments, fast setup, and developer workflows.

Recommended schema areas:
- `assets`
- `services`
- `devices`
- `sensors`
- `observations`
- `agent_results`
- `assessments`
- `reports`
- `audit_logs`
- `users`

### Upgrade path: PostgreSQL

Structure the schema and repository layer so the application can switch from SQLite to PostgreSQL with minimal code changes.

Design principles:
- Query via repository abstractions instead of direct on-DB logic
- Use database-agnostic models in shared library
- Use connection configuration and environment-based selection
- Keep migrations explicit and versioned

Recommended configuration:
- `DATABASE_URL=sqlite:///data/pqc_assessment.db` by default
- `DATABASE_URL=postgresql://...` for enterprise deployments

---

## 8. Assessment model

The backend runs rule-based PQC readiness checks against all normalized evidence.

Examples of assessment categories:
- TLS library version support
- SSH protocol policy
- Certificate and key algorithm strength
- Key rotation and certificate expiry status
- Weak legacy crypto configuration
- Mixed or unsupported hybrid deployments
- Public key distribution and trust posture

Assessment outputs:
- Ready / At Risk / Needs Attention / Unknown
- Risk score
- Evidence references
- Suggested remediation

The library should provide a rule catalog with:
- Rule ID
- Description
- Severity
- Condition logic
- Remediation guidance

---

## 9. Logging and audit

The backend must record all important activity:
- Sensor runs
- Agent uploads
- User actions in web UI
- Assessment execution
- Report generation
- Configuration changes
- Credential access events

Audit log requirements:
- Timestamp
- Actor type: user, sensor, system
- Action performed
- Result status
- Related asset or device

This is important both for compliance and for operational troubleshooting.

---

## 10. Security and privacy requirements

- Use encrypted transport for agent-to-backend communication
- Minimize credential storage; prefer temporary secret brokers or secure vaults
- Restrict root-based operations to the supervised local mode
- Redact sensitive values in user-facing logs
- Limit access by role (admin, auditor, operator)
- Require explicit consent for network scans in scope-defined environments

---

## 11. Suggested implementation strategy

### Phase 1: Foundation
- Set up monorepo structure
- Implement shared model library
- Add SQLite repository layer
- Create backend API skeleton
- Build web dashboard shell

### Phase 2: Passive network sensor
- Implement TLS and SSH traffic capture logic
- Add passive asset discovery and session classification
- Send events to backend

### Phase 3: Device agent
- Implement CSV export mode
- Add local config collection for OpenSSL/OpenSSH
- Add upload pipeline to frontend/backend

### Phase 4: Remote and discovery sensors
- Add credentialed remote checks
- Add safe discovery scanning
- Connect all sensors to backend orchestration

### Phase 5: Assessment and reporting
- Implement scoring engine
- Build reports and dashboards
- Add audit logging and role-based access

---

## 12. Recommended folder layout for active work

```text
pqc-assessment/
├── apps/
│   ├── web/
│   │   ├── src/
│   │   ├── public/
│   │   └── README.md
│   └── backend/
│       ├── src/
│       ├── migrations/
│       └── README.md
├── libs/
│   └── common/
│       ├── src/
│       └── README.md
├── sensors/
│   ├── network/
│   │   └── README.md
│   ├── device/
│   │   └── README.md
│   ├── remote/
│   │   └── README.md
│   └── discovery/
│       └── README.md
├── devices/
│   └── README.md
├── docs/
│   └── architecture.md
├── data/
├── scripts/
├── Cargo.toml
├── README.md
└── .gitignore
```

---

## 13. Final recommendation

This should be implemented as a modular, multi-component system centered around a backend orchestrator and a shared library. The sensor components should remain independent and specialized. The database should start with SQLite but be abstracted so PostgreSQL can be introduced later without rewriting the domain logic.

This architecture gives a clean separation between:
- network visibility
- endpoint configuration evidence
- remote validation
- discovery
- aggregation and assessment
- reporting and administration

That separation makes the system easier to extend, easier to test, and easier to modify component-by-component as requirements evolve.
