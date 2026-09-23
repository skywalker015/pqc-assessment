# PQC Readiness Platform: Implementation Plan

This document outlines the step-by-step implementation plan for the PQC Readiness Assessment platform, based on the architecture and detailed design documents. The project will be developed using Rust in a monorepo structure, utilizing SQLite as the default database with a clear migration path to PostgreSQL.

## Phase 1: Foundation
**Goal:** Establish the core workspace, shared data models, database repository layer, and the basic backend/frontend shell.

- **Tasks:**
  - Initialize the Rust workspace (`Cargo.toml`) and directory structure (`apps/`, `libs/`, `sensors/`, `devices/`).
  - Implement the shared domain models in `libs/common/` (Asset, Service, DeviceConfigEvidence, Assessment).
  - Set up the SQLite database schema and repository interfaces in `libs/common/db/`.
  - Scaffold the backend API (`apps/backend/`) with health check endpoints.
  - Build the basic web dashboard shell (`apps/web/`).
  - Create parsers for JSON and CSV data ingestion.

## Phase 2: Sensor Core & Passive Observation
**Goal:** Develop the network and discovery sensors to safely identify assets and intercept TLS/SSH traffic metadata.

- **Tasks:**
  - Implement the **Network Sensor** (`sensors/network/`) to passively capture packets and analyze TLS handshakes and SSH banners.
  - Implement the **Discovery Sensor** (`sensors/discovery/`) to perform ping sweeps and non-intrusive Nmap scans.
  - Build backend API endpoints to receive telemetry and observation data from these sensors.
  - Map incoming data to `Observation`, `Service`, and `AssetCandidate` domain objects and store them in the database.

## Phase 3: Device & Remote Agents
**Goal:** Enable local and remote collection of cryptographic configuration evidence (e.g., OpenSSL/OpenSSH versions, keys, TLS policies).

- **Tasks:**
  - Implement the **Device Agent** (`sensors/device/`) with a supervised mode to export CSV files containing local config evidence.
  - Add unsupervised mode to the Device Agent to push JSON payloads directly to the backend API via scheduled tasks.
  - Implement the **Remote Agent** (`sensors/remote/`) to perform credentialed SSH logins and remote evidence collection.
  - Create the CSV upload pipeline in the web frontend and the corresponding processing logic in the backend.

## Phase 4: Assessment & Reporting
**Goal:** Evaluate normalized evidence against PQC readiness rules, score the environment, and present the results.

- **Tasks:**
  - Build the Rule Engine in `libs/common/rules/pqc_rules.rs` to evaluate PQC compliance (e.g., legacy crypto configs, key strengths).
  - Implement the Assessment Engine in the backend to trigger evaluations on new data and generate risk scores and findings.
  - Develop reporting modules to summarize findings and suggest remediations.
  - Build web dashboard widgets to visualize the overall readiness score, assessment details, and evidence trails.

## Phase 5: Hardening & Enterprise Readiness
**Goal:** Secure the platform, implement robust logging, and ensure it can scale to enterprise deployments.

- **Tasks:**
  - Implement Role-Based Access Control (RBAC) and authentication for the web UI and API.
  - Secure sensor-to-backend communications (TLS/mTLS) and implement strict secret management for remote credentials.
  - Build the centralized audit logging subsystem to track user actions, sensor runs, and assessment generations.
  - Prepare and test the database migration path to support PostgreSQL for enterprise deployments.
  - Finalize end-to-end testing, including integration tests for sensor ingestion and workflow simulations.

## Deployment Strategy
- **Initial Pilot:** Deploy locally using SQLite, with backend and web UI on the same host, gathering data via supervised CSV uploads and passive network sensors.
- **Enterprise Rollout:** Transition to PostgreSQL, distribute sensors across network segments, and utilize remote/unsupervised device agents with central orchestration and audit logging.
