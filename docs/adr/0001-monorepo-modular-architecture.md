# ADR 0001: Monorepo with modular backend and sensor components

- Status: Accepted
- Date: 2026-09-23

## Context

The project needs to support multiple distinct discovery and validation flows: passive network observation, local device inspection, remote validation, and discovery scanning. These flows share a common domain model and assessment engine but are operationally distinct.

## Decision

The project will use a Rust monorepo with a shared domain library and separate subsystem crates for backend orchestration and sensor execution.

## Rationale

- Clear separation of responsibility between orchestration, domain logic, and evidence collection
- Easier code ownership and parallel implementation by subsystem
- Lower risk of cross-coupling between local-device checks and network monitoring
- Allows gradual expansion without forcing a single large service from day one

## Consequences

### Positive

- Better modularity and testability
- Cleaner dependency boundaries
- Easier staged implementation

### Negative

- Requires deliberate dependency management and contract discipline
- Needs a shared contract for sensor payloads and domain types

## Follow-up

The crate dependency map should enforce that only the shared domain layer owns cross-cutting models and schema definitions.
