# Security Architecture

## 1. Purpose

This document defines the security boundaries, trust assumptions, and implementation principles for the PQC readiness platform before production deployment. The system handles cryptographic evidence, device metadata, and potentially sensitive configuration data, so security must be designed in from the first implementation sprint.

---

## 2. Security principles

1. Least privilege for sensors and services.
2. Explicit trust boundaries between network, device, remote, and backend components.
3. Strong identity for every sensor and operator action.
4. Secret handling through secure stores, not environment variables alone.
5. Auditability for every assessment, ingestion, and administrative action.
6. Defense in depth for credentialed remote checks and file uploads.
7. Default denial for untrusted data and unsafe network behaviors.

---

## 3. Trust model

### 3.1 Core trust zones

- Public or untrusted network segments
- Device and sensor execution environments
- Backend application tier
- Storage tier (SQLite/PostgreSQL)
- Operator/admin interface
- External secret manager or token-management infrastructure

### 3.2 Trust assumptions

- Network sensors may only gather passive or approved active evidence.
- Remote sensors operate with explicit credentials and must not persist secrets beyond runtime scope.
- Device agents may inspect local configuration but must avoid writing unapproved artifacts outside their designated output path.
- The backend is the authoritative evaluation engine and must validate every inbound payload before storing or scoring it.

---

## 4. Identity and access

### Sensor identity

Every sensor should have a unique, scoped identity:

- `sensor_id`
- issued sensor API token or key identifier
- environment tag
- allowed role (network, device, remote, discovery)
- expiration, rotation, and revocation metadata

### Authentication

Recommended model:

- HTTPS with TLS 1.3 for backend-sensor traffic
- one unique, scoped API token or key per sensor and environment
- short expiration, scheduled rotation, and immediate revocation on compromise
- `Authorization: Bearer <token>` for sensor API requests
- separate tokens for registration, telemetry ingestion, and administrative operations where practical

### Authorization

The backend should enforce role-based access:

- admin: full config, assessment, report, and sensor lifecycle management
- operator: run scans, review reports, inspect logs
- auditor: read-only access to assessment history and evidence trails
- sensor: ingest-only access to assigned telemetry routes

---

## 5. Data sensitivity classification

### Public data
- dashboard overview summaries
- non-sensitive asset metadata
- aggregate risk scores

### Internal data
- environment inventory
- service metadata
- credential metadata and certificate fingerprints
- scan schedules

### Restricted data
- credential material
- private key material
- certificate chains
- raw device config exports
- full telemetry or packet-capture derived artifacts

Restricted data must be encrypted at rest, access-controlled, and subject to retention rules.

---

## 6. Transport and crypto posture

The platform should prioritize modern crypto posture and be designed to remain compatible with post-quantum transitions.

Required practices:

- TLS 1.3 only for external and internal API traffic in production
- scoped API-token authentication on sensor-to-backend channels
- constant-time token comparison and rate limiting on authentication failures
- no shared global sensor token across environments
- cryptographic metadata capture for analyzed services and endpoints
- support for PQC-ready algorithm detection and reporting without depending on it for all legacy systems

The project should be explicit that legacy algorithms remain visible and scored, while PQC-capable algorithms are identified as preferred state.

---

## 7. Secret management

Secrets must never be embedded directly in code, config files, or logs.

Recommended model:

- use a vault or secret manager for credentials and sensor API tokens
- inject runtime values via environment variables or ephemeral mounts
- rotate keys and tokens on a regular schedule
- redact all secret values from logs, metrics, and exception traces

Examples of secret-bearing data:

- SSH private keys
- remote admin credentials
- API tokens
- sensor API tokens and keys
- database connection strings with credentials

---

## 8. Input validation and ingestion security

Every system boundary must validate input before use.

Required checks:

- strict schema validation for JSON and CSV payloads
- size limits for uploads and telemetry payloads
- allow-listing for supported sensor message types
- rejection of malformed or repeated evidence entries
- sanitization before display in audit or dashboard views
- explicit error handling without full stack traces in production logs

Inbound data should be treated as untrusted until validated and normalized by the backend.

---

## 9. Logging, evidence, and audit trails

The platform must maintain an audit trail for all material changes.

Audit events should include:

- sensor registration or re-registration
- assessment run
- rule evaluation result
- configuration changes
- credential rotation
- user actions affecting security posture
- data retention events

Logs must remain tamper-evident enough for operational review and incident response. If the implementation uses a database, the platform should store immutable or append-only records for security-relevant events.

---

## 10. Secure deployment controls

### Deployment boundaries

- backend in a protected control plane
- sensor traffic isolated by network trust segment
- separate secret store and token-management controls
- production DB isolated from developer systems

### Runtime protections

- container or process-level read-only filesystem where possible
- non-root service accounts
- minimal network exposure
- explicit health endpoint with limited operational detail
- security headers and rate limits on public API endpoints

---

## 11. Threats to address

### Common design risks

- credential leakage in logs or telemetry
- sensor spoofing via absent or weak identity checks
- malformed CSV or JSON causing crashes or rules bypass
- excessive privilege on remote device checks
- unbounded data growth from raw ingestion artifacts
- unsafe retention of old observation records

### Mitigations

- scoped tokens, rotation, revocation, and access control
- strict payload validation and schema enforcement
- limited privilege remote execution
- encrypted storage and secure key rotation
- retention pruning and secure archival
- periodic security review and dependency update review

---

## 12. Security review gates

The project should not proceed past prototype stage without a pass on:

- sensor identity and trust model
- secrets storage and rotation
- payload validation and denial-of-service guardrails
- audit trails for security events
- retention and deletion policy
- production deployment hardening

---

## 13. Security acceptance criteria

The feature or service is only considered production-ready when all of the following are true:

- sensor communication uses authenticated and encrypted transport
- credentials are never stored in plaintext
- uploads and ingestion endpoints reject invalid payloads
- audit events exist for changes to assessments and configuration
- no secrets are exposed in logs or error output
- retention and deletion are documented and enforced

---
