# ADR 0003: WireGuard and certificate-based sensor identity

- Status: Superseded by revised enrollment design
- Date: 2026-09-23

## Context

Sensors collect cryptographic evidence and remote configuration data. Without strong identity and encryption, the platform cannot reliably distinguish valid evidence from spoofed or malicious submissions.

## Decision

The revised design uses TLS 1.3 directly for current sensor transport. WireGuard is deferred as an optional future network-isolation profile. Initial enrollment uses server-authenticated TLS without mTLS. Sensors generate keys and CSRs locally; an issuer CA signs approved requests and issues configurable 30-day end-entity certificates. Renewal starts seven days before expiry. OCSP and CRL checks are not used; backend sensor lifecycle state controls access.

## Rationale

- Prevents spoofing of sensor traffic and data injection
- Gives the backend a clear identity model for trusted evidence sources
- Supports secure, auditable sensor registration and certificate renewal
- Fits operational security expectations for enterprise environments

## Consequences

### Positive

- Strong trust boundaries between devices, sensors, and backend
- Better auditability and easier incident handling
- Safer remote credentialed collection paths

### Negative

- Requires PKI certificate-management workflow
- Adds operational complexity for certificate renewal and expired-sensor re-enrollment

## Follow-up

The security architecture and operations runbook define certificate issuance, expiry monitoring, renewal, deletion, and new-sensor re-enrollment before production rollout.
