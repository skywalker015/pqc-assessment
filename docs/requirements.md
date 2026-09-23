# PQC Assessment Requirements and Scope

## 1. Purpose

This project delivers a Rust-based assessment platform for measuring an environment's Post-Quantum Cryptography (PQC) readiness across network services, managed devices, and remote endpoints. The system collects evidence, normalizes observations, applies rules, and produces operational readiness reports.

The implementation target is a Rust workspace with the following crates already defined in the repository:

- `libs/common`
- `apps/backend`
- `sensors/network`
- `sensors/device`
- `sensors/remote`
- `sensors/discovery`

This document defines the product baseline, MVP scope, constraints, and acceptance criteria for the first implementation phase.

The product is an assessment and progress-tracking platform. It determines which in-scope enterprise assets already demonstrate PQC-ready posture, which assets do not, what evidence supports that conclusion, and how mitigation work progresses over time.

---

## 2. Business goals

The platform must help an enterprise answer four operational questions:

1. Which assets and services are in the assessment scope?
2. Which assets already demonstrate a PQC-ready security posture?
3. Which assets are not yet PQC-ready, or cannot yet be classified because evidence is missing?
4. How is remediation progressing across assets, services, teams, and reporting periods?

The primary business outcome is a comprehensive, evidence-backed summary of enterprise asset posture and mitigation progress. The output must support prioritization and reporting, not merely raw discovery.

## 3. Product goals

### Primary goals
- Inventory devices, services, and observed crypto-capable endpoints
- Detect TLS and SSH exposure patterns relevant to PQC migration
- Collect local device-level evidence for crypto configuration
- Run assessment rules against observed evidence
- Produce readiness scores and remediation recommendations
- Classify each in-scope asset as PQC-ready, partially ready, not PQC-ready, or unknown
- Track mitigation status and posture changes between assessment periods
- Provide a comprehensive environment summary showing ready, not-ready, unknown, and remediated assets
- Support both local SQLite usage and PostgreSQL enterprise transition

### Secondary goals
- Support non-intrusive discovery and limited remote credential checks
- Maintain auditability for all sensor and user actions
- Keep secret material out of persistent storage wherever possible

---

## 4. Non-goals for v1

The initial release does not include:
- a dedicated browser-first web application crate in the workspace
- full SaaS multi-tenant orchestration
- advanced ML-driven risk prediction
- broad enterprise IAM and SSO integration
- native mobile apps
- full PQC key exchange implementation as a cryptographic library; the platform focuses on assessment, not cryptographic implementation
- implementing, installing, upgrading, or configuring PQC algorithms on customer assets
- acting as a certificate authority, key management system, or cryptographic migration executor
- claiming that an asset is compliant solely because it has a current software version; readiness must be supported by collected evidence

---

## 5. Scope boundaries

### 6.1 What counts as PQC readiness

For this product, **PQC readiness** means the assessed security posture of an in-scope asset or service, based on available evidence about:

- whether supported PQC or hybrid cryptographic mechanisms are observed or configured
- whether protocol versions and key-exchange choices meet the applicable rule baseline
- whether deployed libraries and services are capable of the required migration path
- whether key, certificate, and trust-store posture contains known blockers
- whether the asset has an owner, evidence timestamp, and an actionable mitigation state

Readiness is an assessment result, not proof that the asset is fully migrated to PQC. The system must distinguish direct evidence, inferred evidence, and missing evidence.

### 6.2 Readiness states

- **PQC-ready:** required checks pass with sufficient evidence for the configured policy.
- **Partially ready:** some checks pass, but one or more material controls remain incomplete or unverified.
- **Not PQC-ready:** one or more material controls fail.
- **Unknown:** the platform lacks sufficient current evidence to classify the asset.

### 6.3 In-scope assets

The assessment scope includes enterprise assets and services that are explicitly discovered, registered, uploaded, or assigned to an authorized sensor scope. Examples include servers, workstations, network appliances, gateways, mail relays, VPN endpoints, and TLS/SSH services.

An asset outside the configured scope must not affect the enterprise readiness score. It may be retained as a discovered candidate until an operator includes or excludes it.

### 6.4 Out-of-scope conclusions

The platform does not determine whether an organization has completed a full PQC migration, does not modify assets, and does not replace a formal compliance audit. Results are limited to the evidence collected by the configured sensors and the versioned rule catalog.

## 6. In-scope functionality

### 4.1 Asset discovery
- Detect network devices and services via passive observations and light discovery scans
- Capture IP addresses, ports, host metadata, and timestamps
- Maintain a current asset map for the environment

### 4.2 Protocol observation
- Observe HTTP/TLS and SSH usage patterns
- Record TLS versions, cipher metadata, and protocol details where available
- Detect legacy or weak public-key exposure patterns

### 4.3 Device inspection
- Collect local crypto configuration from managed endpoints
- Read TLS library versions and SSH configuration metadata
- Check for weak or outdated crypto policy settings
- Export results in CSV for supervised collection and JSON for direct submission

### 4.4 Remote checks
- Connect to target devices using approved credentials
- Collect version and configuration evidence over the network
- Record credential use without retaining sensitive material in plain text

### 4.5 Assessment engine
- Evaluate evidence against a rule catalog
- Score each asset and the environment overall
- Surface risk findings and remediation guidance
- Assign a readiness state with evidence confidence and assessment timestamp
- Compare the current state with prior assessments to show mitigation progress

### 4.6 Reporting
- Comprehensive summaries by asset, service, device type, readiness state, risk level, and mitigation status
- Trend view across collection periods
- Exportable assessment results in JSON or CSV
- Progress reporting for open, in-progress, mitigated, accepted, and unverifiable findings

---

## 7. User roles

### Administrator
- manages sensors and assessment scope
- reviews findings and remediation priorities
- approves environment-wide policy changes

### Operator
- runs discovery and collection jobs
- validates sensor health
- reviews collection failures and reporting output

### Auditor
- reviews rule execution, audit logs, and evidence provenance
- validates compliance posture for internal or external review

---

## 8. Functional requirements

### FR-1: Data ingestion
The platform must accept telemetry from network, device, and remote sensors in structured JSON or CSV form.

### FR-2: Normalization
Raw observations must be mapped to canonical domain records such as assets, services, observations, and assessments.

### FR-3: Assessment execution
The system must evaluate a configured rule set against normalized evidence to produce a per-asset readiness state, risk score, evidence confidence, and remediation recommendations.

### FR-3a: Asset posture summary
The system must identify, for every in-scope asset, whether it is PQC-ready, partially ready, not PQC-ready, or unknown, and must provide the evidence and rule evaluations supporting that classification.

### FR-3b: Mitigation progress
The system must track remediation status for findings and compare assessment periods so operators can report newly remediated assets, remaining gaps, regressions, and stale or missing evidence.

### FR-3c: Scope control
The system must record whether an asset is included, excluded, or pending scope review. Excluded and pending assets must not be silently included in the enterprise readiness score.

### FR-4: Secrets handling
Plaintext credentials must not be stored in the database. Secrets must be retrieved through secure runtime handling or encrypted storage.

### FR-5: Auditability
The system must log sensor submissions, configuration updates, assessment runs, and user actions.

### FR-6: Extensibility
Rules, sensors, and report templates must be configurable without requiring a full application rewrite.

### FR-7: Storage portability
The data model must support SQLite for local/default use and PostgreSQL for enterprise deployment without large domain rewrites.

---

## 9. Non-functional requirements

### NFR-1: Security
- all sensor ingestion endpoints must authenticate using a scoped, revocable API token or API key
- data in transit must be encrypted with TLS 1.3 and modern safe defaults
- credential material must be protected at rest and in memory

### NFR-2: Reliability
- ingestion should tolerate partial data loss and malformed payloads gracefully
- sensor failures must not block the entire assessment engine

### NFR-3: Performance
- telemetry ingestion must support bursty sensor traffic
- assessment runs must not require reprocessing the entire dataset for every small update

### NFR-4: Observability
- health endpoints, structured logs, and run status must be available
- sensor heartbeat and last-seen timestamps must be tracked

### NFR-5: Maintainability
- modules should remain loosely coupled and aligned to the Rust workspace structure
- shared domain types must live in `libs/common`

---

## 10. Explicit MVP success criteria

The MVP succeeds when an authorized operator can produce an evidence-backed answer to: “Which in-scope enterprise assets need PQC remediation, which already demonstrate PQC readiness, and how is mitigation progressing?”

The MVP is complete when all of the following are true:

1. A Rust workspace builds successfully using the project structure defined in `Cargo.toml`.
2. The backend can ingest telemetry from at least one sensor type.
3. The shared library contains canonical domain models for asset, service, observation, and assessment.
4. The rule engine evaluates at least the core PQC rules from the rules catalog.
5. A local SQLite-backed installation can run without external service dependencies.
6. The platform produces an overall readiness summary plus per-asset readiness states.
7. Secrets are not persisted in plain text.
8. Sensor and assessment activity are recorded in audit logs.
9. The project includes a clear path to PostgreSQL deployment for enterprise scale.
10. Each readiness state is backed by rule evaluations, evidence references, timestamps, and an evidence-confidence value.
11. Operators can distinguish PQC-ready, partially ready, not PQC-ready, unknown, excluded, and pending-scope assets.
12. Operators can record and report mitigation status and compare it across at least two assessment periods.
13. A test fixture demonstrates that a known-ready asset is classified as ready and a known-non-ready asset is classified as not ready.

---

## 11. Deployment assumptions

- Local default: SQLite
- Target enterprise upgrade: PostgreSQL
- Sensor-to-backend communication: HTTPS with scoped API-token authentication and authenticated payloads
- Assessment execution: backend-driven, triggered after ingestion or on schedule
- User interface: initially minimal operational and reporting surfaces; full browser app is a later milestone

---

## 12. Risks and constraints

### Key risks
- telemetry flood from passive sensors
- secrets leakage from remote or device collection
- inconsistent schema across sensor payloads
- false positives in weak crypto detection

### Key constraints
- do not perform destructive or intrusive scanning by default
- do not persist credentials in plaintext
- keep rule evaluation explainable and auditable
- prefer safe, non-disruptive asset discovery in the early deployment phase

---

## 13. Definition of done for the project phase

The project phase is considered complete when:
- the workspace builds cleanly,
- sensor ingestion works end-to-end,
- data persists correctly in SQLite,
- assessments produce actionable findings,
- security assumptions are documented and enforceable,
- and execution can be repeated locally by a developer or operator using documented setup steps.
