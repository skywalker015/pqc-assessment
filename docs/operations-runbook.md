# Operations Runbook

## 1. Purpose

This runbook defines the day-to-day operational procedures for a pilot or staged enterprise deployment of the PQC readiness platform. It is meant to be practical and implementation-oriented so the platform can be maintained without relying on tribal knowledge.

---

## 2. Operational roles

- Platform operator: validates deployment health and responds to incidents
- Sensor owner: manages sensor registration, rotation, and configuration changes
- Security reviewer: validates credential and certificate posture
- Auditor: reviews logs, evidence retention, and historical results

---

## 3. Standard startup checklist

Before a backend or sensor deployment starts, verify:

1. the environment is set correctly (`APP_ENV`, `DATABASE_URL`, etc.)
2. TLS certificate and key paths are present and valid
3. the database is reachable and initialized
4. the secret store is reachable and the runtime role is authorized
5. sensor certificates are installed and not expired
6. health endpoints can be reached
7. the log sink and metrics namespace are configured

---

## 4. Health checks

### Backend health

The backend should expose a minimal health endpoint and status summary such as:

- `GET /health`
- database status
- sensor heartbeats
- assessment engine status
- process uptime
- last successful ingestion timestamp

### Sensor health

For each sensor, verify:

- last-seen timestamp is within acceptable thresholds
- registration is valid
- certificate is not expired
- payload ingestion is successful
- result summaries are emitted without malformed data

### Alert thresholds

Recommended default thresholds:

- sensor silent for > 15 minutes = warning
- sensor silent for > 1 hour = critical
- backend DB errors = critical
- repeated failed assessment jobs = warning or critical depending on severity

---

## 5. Daily operations

### Daily checks

- confirm all expected sensors are active
- review failed or delayed ingestion jobs
- confirm database size and retention jobs are operating normally
- verify assessment runs are completing successfully
- review audit logs for configuration changes, credential rotation, and admin actions

### Weekly checks

- inspect sensor certificate validity windows
- confirm secret rotation schedule remains on track
- review report generation quality and sample outputs
- validate stale assets or orphaned sensors are cleared
- inspect storage volume trends and retention execution

---

## 6. Incident response

### 6.1 Sensor outage

Symptoms:
- no heartbeat from a sensor
- repeated ingestion failures
- stale asset updates

Procedure:

1. confirm the sensor host is online
2. check certificate validity and trust chain
3. verify backend connectivity and endpoint reachability
4. inspect timestamp and payload errors in the audit log
5. validate credentials or secret retrieval path
6. restore service or rotate certificates if required
7. document the incident and postmortem actions

### 6.2 Backend failure

Symptoms:
- health endpoint fails
- DB connection errors
- assessment jobs queued but not running

Procedure:

1. confirm service process state
2. review logs for config drift or dependency failures
3. verify DB connectivity and migration status
4. restore last-known-good config and restart service
5. validate health endpoints and one test ingestion
6. if necessary, restore from backup and replay supported jobs

### 6.3 Credential or certificate issue

Symptoms:
- mTLS handshake failure
- remote sensor cannot authenticate
- secret retrieval failures

Procedure:

1. verify CA chain and certificate validity
2. rotate expired or invalid credentials
3. confirm runtime secret injection path
4. validate new credentials on a non-production sensor before rollout
5. update the audit record and notify affected operators

---

## 7. Deployment and configuration changes

Changes should be staged and reviewed before release.

Required change workflow:

1. create change record or ADR for architecture-impacting changes
2. validate new config in a lower environment
3. confirm secrets and certificates are rotated only as needed
4. deploy one sensor or backend group at a time
5. validate health after rollout
6. keep rollback script, config snapshot, and DB backup available

---

## 8. Backup and restore

The deployment should maintain at least:

- DB backup or snapshot strategy
- last-known-good config bundle
- certificate bundle archive
- secret configuration metadata
- staging or rollback environment for sensor test

Recommended restore sequence:

1. halt or isolate affected component
2. restore config and secrets from the known-good bundle
3. restore database snapshot if necessary
4. bring service back online
5. validate health endpoints and ingestion
6. re-enable sensors gradually

---

## 9. Data hygiene and operational cleanup

Operational tasks should include:

- pruning stale results according to policy
- removing orphaned sensor records
- reprocessing failed ingestion jobs after root cause is fixed
- archiving reports or raw evidence as needed
- reviewing duplicate or conflicting asset entries

---

## 10. Maintenance windows

Use a defined maintenance window for:

- certificate rotation
- database migration or vacuum operations
- sensor upgrades
- rule and scoring changes
- environment-level configuration updates

No major change should be applied without:

- a recorded release note
- a health check before/after rollout
- a documented rollback path

---

## 11. Escalation and ownership

- sensor owner responsible for device or network sensor issues
- backend operator responsible for service health and DB concerns
- security reviewer responsible for mTLS, credentials, and secret issues
- engineering lead responsible for architecture-level or rule-level changes

---

## 12. Exit criteria for operational readiness

A deployment is considered operationally ready when:

- health checks pass consistently
- sensor heartbeat expectations are met
- config, secret, and certificate paths are validated
- ingestion and assessment jobs complete without critical error
- retention and backup processes have been run successfully
- a rollback procedure has been tested at least once

---
