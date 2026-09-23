# PQC Assessment Deployment Guide

## 1. Deployment goals

The platform is designed for incremental deployment, starting with a local SQLite-backed pilot and scaling into enterprise deployments with PostgreSQL, multiple sensors, and stronger secret-handling controls.

This document defines the expected deployment patterns for the current repo structure and the first implementation phases.

---

## 2. Deployment topology overview

### 2.1 Local pilot topology

Use this for developer testing and small environments.

```text
+----------------------+       +---------------------+
| Developer/Operator   | ----> | Backend API         |
| workstation          |       | apps/backend        |
+----------------------+       +----------+----------+
                                             |
                                             v
                                    +-------------------+
                                    | SQLite database   |
                                    | local file store  |
                                    +-------------------+

+----------------------+       +----------------------+
| Discovery / network  | ----> | Backend API          |
| device sensor        |       | ingest endpoint      |
+----------------------+       +----------------------+
```

This topology keeps the system simple:
- one backend host
- one local database
- one or more sensors sending data over HTTPS

---

### 2.2 Enterprise topology

Use this for a larger assessment environment with multiple network segments or sites.

```text
+-----------------+   +-----------------+   +-----------------+
| Network Sensor  |   | Device Sensor   |   | Remote Sensor   |
+--------+--------+   +--------+--------+   +--------+--------+
         |                     |                      |
         +---------+-----------+----------------------+---------+
                   |
                   v
              +----------------------+
              | Backend API / Core   |
              | apps/backend         |
              +----------+-----------+
                         |
                         v
              +----------------------+
              | PostgreSQL           |
              | enterprise storage   |
              +----------------------+

              +----------------------+
              | Secret store / token |
              | management          |
              +----------------------+
```

---

## 3. Runtime components

### Backend service
- primary system of record
- ingests telemetry and assessment results
- stores normalized records
- executes assessment rules
- exposes results and health endpoints

### Sensor services
- network sensor: event and protocol observations
- discovery sensor: asset and service discovery
- device sensor: endpoint-level crypto configuration
- remote sensor: credentialed remote validation

### Shared domain library
- canonical models and rule definitions
- validation and parser helpers
- report generation primitives

---

## 4. Environment configuration

The backend should read secrets and runtime config from environment variables or a secure secret manager. Recommended keys include:

- `APP_ENV`
- `DATABASE_URL`
- `APP_BIND_HOST`
- `APP_BIND_PORT`
- `SECRET_STORE_URL`
- `SENSOR_TOKEN_STORE_URL`
- `JWT_SECRET`

Example local configuration:

```env
APP_ENV=local
DATABASE_URL=sqlite:///./data/pqc-assessment.db
APP_BIND_HOST=0.0.0.0
APP_BIND_PORT=8080
```

Example enterprise configuration:

```env
APP_ENV=prod
DATABASE_URL=postgresql://user:pass@db:5432/pqc_assessment
APP_BIND_HOST=0.0.0.0
APP_BIND_PORT=8080
SECRET_STORE_URL=https://vault.internal
SENSOR_TOKEN_STORE_URL=https://vault.internal/tokens
```

---

## 5. Security deployment requirements

### API tokens for sensors
- every sensor should authenticate with a unique, scoped API token or key
- sensors should not share a single static token across environments
- token expiration, rotation, and revocation should be automated or scheduled

### Secrets handling
- no plaintext credentials in the database
- retrieve secrets from a secure runtime vault or encrypted secret store
- redact secrets from logs and audit output

### Transport security
- use HTTPS with modern TLS 1.3 defaults
- prefer PQC-ready algorithms in upstream environments where supported

---

## 6. Database strategy

### Local default
- use SQLite for development and pilot deployments
- store the database under a dedicated `data/` directory or explicit local path

### Production upgrade path
- move to PostgreSQL for larger installations or multi-sensor environments
- keep the application layer isolated from storage-specific logic when possible

---

## 7. Health and operational checks

The backend should expose health and status diagnostics such as:

- `GET /health`
- `GET /api/version`
- sensor heartbeat status
- database connectivity status
- data retention job status

A deployment should be considered healthy only when:
- backend process is running,
- database connection is healthy,
- sensor heartbeats are recent,
- rule engine is available,
- and assessment jobs can complete without critical failures.

---

## 8. Deployment phases

### Phase 1: developer environment
- SQLite database
- single backend process
- one or more local sensors
- manual configuration and policy review

### Phase 2: pilot environment
- local or private network deployment
- one shared backend instance
- limited set of sensors in a controlled segment
- formal audit log review

### Phase 3: enterprise rollout
- PostgreSQL backend
- segmented sensor architecture
- token rotation, revocation, and secret storage
- centralized reporting and operator controls

---

## 9. Rollback and recovery

For each deployment, keep:
- previous database backup
- last-known-good config files
- last-known-good token configuration metadata
- known rollback procedure for backend and sensor versions

Recovery plan:
1. stop or isolate the affected backend or sensor group,
2. restore last known good configuration,
3. restore DB if needed,
4. verify health endpoints,
5. resume sensor traffic gradually.

---

## 10. Operational readiness checklist

Before deployment, ensure:
- sensor API tokens are scoped, valid, and not expired
- token authentication and revocation checks succeed
- database connection strings are correct
- secret store is available
- audit logging is enabled
- retention jobs are scheduled
- health checks are reachable
- rollback plan is documented

---

## 11. Success criteria

The deployment is considered ready when:
- the backend can start cleanly,
- sensors can authenticate and send telemetry,
- data persists in the chosen database,
- assessment runs complete and surface findings,
- operators can observe health, risk, and audit state,
- and the environment can be recovered without uncontrolled downtime.
