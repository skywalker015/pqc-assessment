# PQC Readiness Platform Threat Model

This document outlines the security threat model for the platform using the STRIDE methodology.

## 1. Trust Boundaries
- **Unsupervised Sensors -> Backend:** Untrusted network, secured by TLS and scoped API-token authentication.
- **Web UI -> Backend:** Untrusted network, secured by JWT and standard TLS.
- **Backend -> Database:** Trusted network, secure local or private subnet connection.
- **Remote Sensor -> Target Device:** Untrusted network, secured via SSH.

## 2. STRIDE Analysis

### Spoofing
- **Threat:** A malicious actor impersonates a sensor to submit fake compliance data (e.g., claiming a vulnerable asset is PQC-ready).
- **Mitigation:** Every sensor receives a unique, scoped, expiring API token or key. Ingestion endpoints enforce token validation, rate limits, rotation, and revocation.

### Tampering
- **Threat:** Network traffic between the sensor and backend is modified in transit.
- **Mitigation:** All data in transit uses PQC-ready encrypted transport (TLS 1.3 with ML-KEM key exchange).

### Repudiation
- **Threat:** An admin modifies the PQC rule engine or deletes a sensor, then denies doing so.
- **Mitigation:** Centralized, append-only audit logging is required for all configuration changes and API access. 

### Information Disclosure
- **Threat:** The remote sensor's SSH credentials for scanning devices are leaked from the database.
- **Mitigation:** Passwords and private keys must never be stored in plain text. The system must use a secure vault (e.g., HashiCorp Vault) or temporary token brokers to retrieve credentials at runtime.

### Denial of Service (DoS)
- **Threat:** Thousands of sensors send telemetry simultaneously, crashing the backend.
- **Mitigation:** The Rust backend uses the **Tokio** async runtime for high concurrency. Data aggregation and rate-limiting are enforced at the API layer.

### Elevation of Privilege
- **Threat:** A user with "Operator" access attempts to view audit logs or generate admin-level reports.
- **Mitigation:** Strict Role-Based Access Control (RBAC) enforced on every API route.

