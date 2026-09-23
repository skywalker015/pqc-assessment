# Data Retention and Pruning Policy

## 1. Purpose

The PQC readiness platform collects operational evidence, telemetry, and assessment records. Because this data can include cryptographic metadata, configuration findings, and potentially sensitive information, the project needs an explicit retention and pruning policy before the first production deployment.

---

## 2. Policy goals

- limit storage growth over time
- avoid indefinite storage of sensitive raw evidence
- preserve enough historical information for assessment and audit
- allow a safe, auditable deletion process
- support legal or compliance review requirements without losing operational continuity

---

## 3. Data categories

### 3.1 Raw evidence
Examples:
- CSV uploads from supervised device agents
- raw observation payloads
- source-level network metadata
- device-local crypto config exports

Default handling:
- retain only for the minimum period needed for operational validation
- keep a reduced or normalized copy for reporting
- delete raw artifacts after successful ingestion and validation unless there is a specific compliance reason to retain them

### 3.2 Normalized observations
Examples:
- parsed service metadata
- protocol records
- certificate summaries
- hash or fingerprint references
- inventory updates

Default handling:
- retain for a longer duration than raw evidence
- preserve enough history for trend analysis and risk evaluation

### 3.3 Assessment results
Examples:
- readiness scores
- rule results
- risk classifications
- report generations

Default handling:
- keep for the longest period needed for audit and historical reporting
- maintain explicit versioning for rule changes so older assessments remain interpretable

### 3.4 Audit records
Examples:
- sensor registration events
- config changes
- admin actions
- certificate rotation

Default handling:
- keep as long as operationally required for governance and security review

---

## 4. Recommended retention periods

These are baseline recommendations for pilot or early production environments:

- raw evidence: 30 to 90 days
- normalized observations: 180 days to 2 years
- assessment results: 1 to 3 years
- audit records: 1 to 7 years depending on compliance and policy requirements

Actual retention should be refined by the deployment environment and sign-off from the project owner.

---

## 5. Pruning workflow

The backend should execute a scheduled pruning job with the following behaviors:

1. identify expired raw evidence records
2. validate whether the normalized records or assessments still need them
3. delete or archive raw data securely
4. record the pruning event in the audit log
5. generate a summary of deleted records and affected environment segments

Pruning must be logged and auditable so it is not an invisible cleanup process.

---

## 6. Redaction and anonymization

Retention must be paired with redaction practices:

- do not store plaintext credentials or private keys
- redact sensitive identifiers in reports if they are not essential
- hash or fingerprint generated values where a stable reference is sufficient
- avoid keeping secrets in evidence or report exports

---

## 7. Archival strategy

If long-term retention is required:

- archive sanitized normalized data rather than raw secret-bearing files
- prefer encrypted archives and access-controlled storage
- keep a clear mapping between archived records and their originating assessments
- maintain versioned retention rules so archival rules remain understandable

---

## 8. Deletion acceptable use

Deletion should happen only when:

- the retention period has expired,
- the data is no longer required for assessment evidence,
- and the deletion is logged and consistent with policy.

For high-risk or regulated environments, legal review may require a longer retention period even when raw evidence exceeds the default baseline.

---

## 9. Retention policy checkpoints

The following must be validated before production release:

- app has a scheduled pruning mechanism
- raw files are removed or archived after ingestion validation
- audit log contains pruning events
- deleted data does not remain in accessible report exports or dashboards
- long-term storage does not create secret leakage risk

---
