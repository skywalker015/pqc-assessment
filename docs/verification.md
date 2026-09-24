# PQC Assessment Verification Strategy

## 1. Objective

This document defines how the project should be verified before claiming the implementation is complete. The goal is to prove that the backend, sensors, and assessment logic behave correctly and safely under realistic conditions.

The verification strategy is intentionally layered:
- unit tests for domain logic
- integration tests for ingestion and persistence
- end-to-end checks for sensor-to-backend flows
- security checks for secrets and audit handling

---

## 2. Verification principles

- write tests before or during implementation for new rules and flows
- prefer real behavior over mock-heavy tests
- verify data integrity, not just happy-path outputs
- test safety and failure handling in addition to success cases
- ensure every assessment includes human-readable evidence

---

## 3. Test categories

### 3.1 Unit tests

Focus on low-level domain logic and rule evaluation.

Examples:
- asset normalization from raw observation payloads
- service classification by protocol and port
- rule evaluation for TLS versions and weak crypto findings
- parsing of CSV and JSON sensor payloads
- status mapping and readiness score generation

Recommended coverage for MVP:
- domain model conversions
- rule matching logic
- assessment summary logic
- parser validation error cases

---

### 3.2 Integration tests

Focus on backend behavior with a real database and serialized inputs.

Examples:
- ingest sensor telemetry to the database
- create linked `Asset` and `Service` records from observation payloads
- store `AssessmentResult` and `RuleEvaluation` objects after rule execution
- verify failed ingestion is logged without corrupting state
- validate retention and pruning behavior

A realistic SQLite integration test should be used before any PostgreSQL migration validation.

---

### 3.3 End-to-end tests

Focus on sensor-to-backend workflow behavior.

Examples:
- network sensor submits telemetry to backend
- remote or device sensor submits CSV or JSON payload
- backend normalizes and stores the payload
- rule engine computes risk score
- report or summary endpoint reflects the new data

End-to-end tests should include at least one failure case such as invalid TLS data or unauthorized sensor submissions.

---

### 3.4 Security verification

The project must explicitly verify:
- no plaintext credentials are written to database logs
- secret files are not accidentally included in reports
- audit log records include important actions without exposing secret material
- unauthorized sensor requests are rejected
- TLS 1.3 server authentication succeeds during enrollment
- current sensor traffic works without WireGuard
- certificate expiry and seven-day renewal behavior are verified
- deleted sensors are denied enrollment, renewal, and telemetry

---

## 4. Required validation gates

The project should only be considered ready after all of the following checks pass:

### Gate 1: Build validation
- workspace compiles successfully with `cargo build`
- no unresolved modules, missing features, or broken workspace members

### Gate 2: Unit validation
- all core domain and rule tests pass
- parsing and scoring behavior is verified under valid and invalid inputs

### Gate 3: Integration validation
- data flows from sensor ingest through normalization to persisted records
- database keys and foreign references remain consistent

### Gate 4: Security validation
- secrets are excluded from logs and reports
- authentication checks reject unauthorized calls

### Gate 5: Operational validation
- health endpoint is available
- database connectivity is stable
- assessment summary output reflects newly submitted data

---

## 5. Example verification scenarios

### Scenario A: TLS mitigation
Input: a service reports a TLS 1.0 handshake with legacy cipher metadata.

Expected result:
- asset and service are created or updated
- relevant observation is stored
- rule `PQC-TLS-001` fails
- assessment result includes risk severity and remediation note

### Scenario B: OpenSSH weakness
Input: remote device reports OpenSSH version below the supported threshold.

Expected result:
- rule `PQC-SSH-001` triggers
- asset is marked at risk
- summary explains the finding and remediation action

### Scenario C: Secret handling
Input: a sensor payload includes a credential or token-like field.

Expected result:
- credential is not stored in plain text
- audit trail records the event without exposing the actual secret
- log output includes a redacted form only

---

## 6. Acceptance criteria for release readiness

The project is ready for a milestone release only if:
- the workspace builds cleanly,
- the core rules are validated with test data,
- ingestion and persistence behave correctly from end to end,
- the system can detect and report risk findings using real sample inputs,
- audit and security checks pass,
- and there is a documented rollback or recovery path for the current deployment mode.

---

## 7. Review checklist for each change

Before merging or closing a change, verify:
- related tests were added or updated
- bad inputs are handled without crash or data corruption
- assessment output remains explainable
- any new rule or sensor field is documented in the rules catalog or API specs
- logs and audit output remain redacted and useful

---

## 8. Operational verification cadence

For each release or environment cutover, perform:
1. clean build
2. sensor ingest smoke test
3. database migration or schema verification
4. sample rule execution
5. security scan for secret exposure
6. health/check endpoint validation
7. review of audit logs

---

## 9. Evidence required before claiming completion

A feature or milestone is only complete when there is fresh verification evidence such as:
- successful `cargo build` output,
- passing targeted test results,
- sample assessment output from a sensor payload,
- audit log entries showing ingestion and rule execution,
- and redaction/security validation for secrets.

This evidence should be retained in the project documentation or CI output for future review.
