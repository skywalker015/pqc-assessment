# Testing Architecture

## 1. Purpose

This document defines the project test strategy for the Rust workspace before implementation begins. It covers unit, integration, rule evaluation, sensor verification, and deployment readiness checks. The goal is not only to verify correctness but to protect the platform against false confidence and silent regressions.

---

## 2. Test strategy principles

1. Write tests at the boundary where behavior matters.
2. Prefer real parsing and validation paths over mock-heavy assertions.
3. Fail fast on invalid ingestion payloads or rule mismatches.
4. Keep assessment logic deterministic and testable.
5. Treat network and sensor behavior as integration concerns, not as pure unit tests.
6. Verify security-sensitive behavior explicitly.

---

## 3. Test layers

### 3.1 Unit tests

Target areas:
- schema validation
- domain model constructors
- rule evaluation logic
- parser and normalization helpers
- certificate and metadata conversion utilities

Unit tests should be fast and deterministic. They should validate domain invariants such as:

- invalid port values are rejected
- TLS metadata is normalized consistently
- asset deduplication is stable
- rule outputs follow expected severity levels

### 3.2 Integration tests

Target areas:

- backend API payload validation
- database persistence and retrieval flows
- end-to-end ingestion of sensor data
- report generation from stored results
- migration and schema compatibility checks

These tests should run against a test database and exercise realistic payloads rather than synthetic mocks only.

### 3.3 Sensor verification tests

Target areas:

- passive network observation parsing
- remote sensor credential handling with safe test fixtures
- device agent CSV export correctness
- discovery result normalization
- invalid or partially missing data handling

Sensor tests should verify the translation from raw evidence into canonical models and ensure the result conforms to the shared domain contract.

### 3.4 Security tests

Target areas:

- secret redaction in logs
- invalid certificate rejection
- malformed upload rejection
- API-token authentication failure behavior
- rate-limit or abuse guardrails on ingest endpoints

Security testing should not be deferred until late stages. It is a first-class verification concern.

### 3.5 End-to-end tests

Target areas:

- sensor registration to assessment run
- device or network findings to report generation
- multi-sensor aggregation into a readiness overview
- user workflow through API and dashboard

E2E tests should be minimal but representative. They should validate the primary user journey rather than every UI detail.

---

## 4. Regression and coverage expectations

The project should require:

- unit tests for all core rule and normalization logic
- integration tests for all ingestion and report generation flows
- regression tests for each bugfix or security fix
- coverage gate for critical modules, not just a global numeric threshold

At minimum, critical code paths should include:

- payload validation
- sensor registration
- rule engine execution
- report generation
- secret handling and redaction

---

## 5. CI pipeline gates

The recommended pre-merge pipeline should include:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`
4. targeted integration tests for backend and sensor ingestion
5. security or lint checks for secret leakage and malformed inputs
6. optional smoke deployment validation in a staging environment

A change should not merge if it fails the unit or integration gate, or if it introduces a regression in the assessment or ingestion workflow.

---

## 6. Test data management

Tests must use dedicated fixtures and examples:

- valid TLS observation samples
- malformed JSON examples
- a representative CSV device export
- a discovery output with partial/duplicate records
- a secure example of certificate metadata

The project should never rely on production-like data or sensitive secrets in unit tests.

---

## 7. Rule-engine validation

The rule engine is central to the project and requires explicit validation tests:

- a rule with valid evidence should score as expected
- a rule with partial evidence should warn or fail as defined
- severity thresholds should map to the correct operational outcome
- rule versioning should remain transparent across assessment runs

These rules should be validated by deterministic fixtures so that scoring changes are easy to review.

---

## 8. Operational verification before release

Before declaring a release candidate ready, run:

- backend health validation
- sensor registration validation
- one sample ingestion round trip
- one assessment run from example data
- report generation and retention validation
- security review of any secret-bearing settings

This is not a substitute for CI; it is the final release sanity check that the code works in the target environment.

---

## 9. Exit criteria for test readiness

The implementation is considered test-ready when:

- each layer has explicit tests
- regression coverage exists for prior bug fixes
- invalid payload and security cases are covered
- sensor-to-backend flow is exercised end-to-end
- CI gates are defined and enforced
- release verification is documented and repeatable

---
