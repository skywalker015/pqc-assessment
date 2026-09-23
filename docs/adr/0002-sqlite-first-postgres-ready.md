# ADR 0002: SQLite-first, PostgreSQL-ready storage design

- Status: Accepted
- Date: 2026-09-23

## Context

The system is meant to be easy to start locally and scalable for enterprise deployment. A single database choice is needed that supports quick local setup without locking the project into a fixed operational model.

## Decision

The default implementation will use SQLite for local and pilot deployments, while the architecture will be designed to support PostgreSQL for enterprise scale through repository abstraction and environment-based configuration.

## Rationale

- Lower setup overhead for local development and proof-of-concept deployment
- Easier onboarding for operator teams and early-stage validation
- Creates a clean upgrade path when scale and concurrency demands increase

## Consequences

### Positive

- Fast local deployment
- Simple developer workflows
- Clear migration path to enterprise DB infrastructure

### Negative

- Requires repository abstraction to avoid SQLite-specific assumptions
- Enterprise migration must be planned and tested explicitly

## Follow-up

The backend should use a storage abstraction layer and explicit migration strategy before PostgreSQL is adopted in production.
