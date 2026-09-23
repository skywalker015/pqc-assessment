# ADR 0003: mTLS-based sensor identity and authenticated communication

- Status: Accepted
- Date: 2026-09-23

## Context

Sensors collect cryptographic evidence and remote configuration data. Without strong identity and encryption, the platform cannot reliably distinguish valid evidence from spoofed or malicious submissions.

## Decision

The project will require authenticated, encrypted communication between sensors and the backend using TLS 1.3 and mutual TLS (mTLS) wherever possible, with scoped certificates and short-lived credential patterns where needed.

## Rationale

- Prevents spoofing of sensor traffic and data injection
- Gives the backend a clear identity model for trusted evidence sources
- Supports secure, auditable sensor registration and rotation
- Fits operational security expectations for enterprise environments

## Consequences

### Positive

- Strong trust boundaries between devices, sensors, and backend
- Better auditability and easier incident handling
- Safer remote credentialed collection paths

### Negative

- Requires PKI or certificate-management workflow
- Adds operational complexity for certificate rotation and renewal

## Follow-up

The security architecture and operations runbook must define certificate issuance, expiry monitoring, and rotation steps before production rollout.
