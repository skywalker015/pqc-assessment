# Project Documentation Index

This directory contains the design, requirements, and implementation-readiness documentation for the PQC assessment system.

## Core documents

- [architecture.md](architecture.md) — system architecture and subsystem breakdown
- [detail-design.md](detail-design.md) — detailed implementation design
- [implementation-plan.md](implementation-plan.md) — staged execution roadmap
- [requirements.md](requirements.md) — functional and non-functional requirements
- [data-model.md](data-model.md) — domain model and storage schema
- [api-specs.md](api-specs.md) — API contracts and interfaces
- [pqc-rules-catalog.md](pqc-rules-catalog.md) — readiness rules and scoring catalog
- [threat-model.md](threat-model.md) — security risk analysis and mitigations
- [security-architecture.md](security-architecture.md) — identity, trust boundaries, secret handling, and design controls
- [deployment.md](deployment.md) — local and enterprise deployment topology
- [operations-runbook.md](operations-runbook.md) — day-to-day operational procedures and incident response
- [data-retention-policy.md](data-retention-policy.md) — retention, pruning, and archival rules
- [testing-architecture.md](testing-architecture.md) — validation strategy and CI expectations
- [verification.md](verification.md) — release and verification gate checklist
- [crate-dependency-map.md](crate-dependency-map.md) — mandated crate boundaries and dependency rules
- [adr/README.md](adr/README.md) — architecture decision records

## Reading order

For a first-time reader, the recommended order is:

1. [requirements.md](requirements.md)
2. [architecture.md](architecture.md)
3. [detail-design.md](detail-design.md)
4. [data-model.md](data-model.md)
5. [api-specs.md](api-specs.md)
6. [implementation-plan.md](implementation-plan.md)
7. [security-architecture.md](security-architecture.md)
8. [deployment.md](deployment.md)
9. [verification.md](verification.md)

## Documentation conventions

- Architecture and design docs define intent and boundaries.
- Requirements and data-model docs define the contract.
- Operational and verification docs define how the system is run and proven.
- ADRs capture the key architecture decisions and the reasons they were chosen.
