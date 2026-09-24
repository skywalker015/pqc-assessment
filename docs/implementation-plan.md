# PQC Readiness Platform: Implementation Plan

This document outlines the step-by-step implementation plan for the PQC Readiness Assessment platform, based on the architecture and detailed design documents. The project will be developed using Rust in a monorepo structure (standardized on the **Tokio** async runtime), utilizing SQLite as the default database with a clear migration path to PostgreSQL.

The implementation target is an assessment and mitigation-progress product. It must classify in-scope enterprise assets by PQC posture and report progress; it must not implement or deploy PQC algorithms on customer assets.

## Phase 1: Foundation & Security Core
**Goal:** Establish the core workspace, shared data models, database repository layer, and the basic backend/frontend shell, with a strong emphasis on PQC-ready security.

- **Tasks:**
  - Initialize the Rust workspace (`Cargo.toml`) and directory structure. Standardize on the **Tokio** async framework.
  - Implement the shared domain models in `libs/common/`.
  - Set up the SQLite database schema and repository interfaces, including a **Data Retention & Pruning** subsystem for handling high-volume telemetry.
  - Implement TLS 1.3 sensor transport; defer WireGuard as an optional future network-isolation profile.
  - Implement server-authenticated TLS enrollment without mTLS, CSR submission, issuer signing, configurable 30-day certificates, seven-day renewal, expiry re-enrollment, and deleted-sensor denial.
  - Scaffold the backend API (`apps/backend/`) with REST and optional gRPC inside the protected path.
  - Build the basic web dashboard shell (`apps/web/`).
  - Define readiness, scope, evidence-confidence, and mitigation-status enums in `libs/common/`.

## Phase 2: Sensor Core & Passive Observation
**Goal:** Develop the network and discovery sensors to safely identify assets and intercept TLS/SSH traffic metadata.

- **Tasks:**
  - Implement the **Network Sensor** (`sensors/network/`) to passively capture packets.
  - Implement the **Discovery Sensor** (`sensors/discovery/`) to perform ping sweeps and non-intrusive Nmap scans.
  - Build backend REST API endpoints to receive telemetry. Implement data aggregation logic to prevent database bloat from high-frequency network observations.
  - Map incoming data to `Observation` and `Service` objects.
  - Ensure discovered assets remain pending scope review until explicitly included or excluded.

## Phase 3: Device Agents & Management
**Goal:** Enable local collection of cryptographic configuration evidence and establish central agent management.

- **Tasks:**
  - Implement the **Agent Management** subsystem in the backend to push configuration updates, polling intervals, and binary upgrades to supervised/unsupervised agents.
  - Implement the **Device Agent** (`sensors/device/`) to poll the backend for updates, and securely push JSON payloads using TLS 1.3 with certificate lifecycle checks.
  - Implement the CSV export/upload pipeline as a fallback supervised mode.
  - Implement the **Remote Agent** (`sensors/remote/`) to perform credentialed SSH logins.

## Phase 4: Dynamic Assessment & Reporting
**Goal:** Evaluate normalized evidence against dynamic PQC readiness rules, score the environment, and present the results.

- **Tasks:**
  - Integrate a **Configurable Rule Engine** (e.g., Rego/OPA or Rhai) in `libs/common/rules/` so PQC assessment rules can be updated dynamically without recompiling the backend.
  - Implement the Assessment Engine to trigger evaluations on new data and generate risk scores.
  - Develop reporting modules to summarize findings and suggest remediations.
  - Implement per-asset classification as PQC-ready, partially ready, not PQC-ready, or unknown.
  - Implement comparison with the previous assessment to report newly remediated assets, open gaps, and regressions.
  - Build web dashboard widgets to visualize the readiness score and rule violations.

## Phase 5: Hardening & Enterprise Readiness
**Goal:** Secure the platform, implement robust logging, and ensure it can scale to enterprise deployments.

- **Tasks:**
  - Implement Role-Based Access Control (RBAC) and authentication for the web UI.
  - Build the centralized audit logging subsystem to track user actions and sensor runs.
  - Prepare and test the database migration path to support PostgreSQL for enterprise deployments.
  - Finalize end-to-end testing, ensuring the pruning jobs successfully maintain database performance and all PQC-ready encryption tunnels hold under load.
  - Verify the explicit success criteria: an operator can identify assets requiring remediation, assets already demonstrating readiness, and mitigation progress across assessment periods.

## Deployment Strategy
- **Initial Pilot:** Deploy locally using SQLite, with backend and web UI on the same host. Sensors communicate using TLS 1.3; initial enrollment uses server-authenticated TLS without mTLS. WireGuard is optional and deferred.
- **Enterprise Rollout:** Transition to PostgreSQL, distribute sensors across network segments, utilize the Agent Management system for fleets of device agents, and rely on the data retention policies to scale efficiently.
