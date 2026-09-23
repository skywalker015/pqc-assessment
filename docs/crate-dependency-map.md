# Crate Dependency Map

## 1. Purpose

This document defines the intended dependency boundaries for the Rust workspace before implementation begins. It is meant to prevent circular dependencies, domain leakage, and over-coupling across the backend, shared library, and sensor crates.

---

## 2. Workspace structure

The repo currently declares the following members:

- `apps/backend`
- `libs/common`
- `sensors/network`
- `sensors/device`
- `sensors/remote`
- `sensors/discovery`

The dependency model should follow a layered architecture:

```text
libs/common
    ^
    |
apps/backend  ->  sensors/*
    ^
    |
web or admin surface (future)
```

In other words:

- the shared domain library should contain the canonical models and rule contracts
- the backend should orchestrate and aggregate
- sensors should emit structured evidence, not own the global system model
- the UI should depend on the backend API, not on sensor internals

---

## 3. Recommended dependency rules

### `libs/common`

Purpose:
- domain types
- protocols and payload schemas
- result models
- rule metadata
- common validation helpers
- serialization helpers

Allowed dependencies:
- no project-local crate should depend on a sensor crate
- should avoid pulling in web or database drivers unless absolutely required

This crate should be the stable baseline for the whole workspace.

### `apps/backend`

Purpose:
- API server
- storage orchestration
- scheduling
- rule execution
- report generation
- sensor lifecycle management

Allowed dependencies:
- `libs/common`
- database client libraries
- optional HTTP framework libraries
- secret management client libraries

Forbidden dependencies:
- direct dependency on the implementation-specific logic of any single sensor crate beyond required interfaces
- business logic that duplicates the domain model from `libs/common`

### `sensors/network`

Purpose:
- passive traffic-based observation and service detection
- proof collection for TLS/SSH behavior and asset discovery

Allowed dependencies:
- `libs/common`
- packet capture or protocol inspection libraries
- local network collection utilities

### `sensors/device`

Purpose:
- local endpoint inspection and environment evidence extraction
- supervised and unsupervised execution modes

Allowed dependencies:
- `libs/common`
- OS/platform inspection crates
- file processing and CSV export utilities

### `sensors/remote`

Purpose:
- credentialed remote validation
- secure remote collection over SSH or approved network channels

Allowed dependencies:
- `libs/common`
- SSH client libraries
- remote execution and secret-scoped credential management

### `sensors/discovery`

Purpose:
- asset inventory and service discovery
- IP and port discovery with limited active probing

Allowed dependencies:
- `libs/common`
- network discovery and scanning primitives

---

## 4. Dependency contract by layer

```text
Layer 1: Platform and infrastructure
- database drivers
- TLS libraries
- secret managers
- logging and tracing
- HTTP server and client crates

Layer 2: Shared domain
- libs/common
- schemas and domain models
- rule metadata and parser utilities

Layer 3: Service orchestration
- apps/backend
- ingestion validators
- report generation
- assessment scheduling

Layer 4: Execution agents
- sensors/network
- sensors/device
- sensors/remote
- sensors/discovery
```

The design should prevent the lower layers from importing the higher layers. Sensors should not import backend types; backend should not import sensor implementation types.

---

## 5. Interface boundaries

Sensors should expose a common contract such as:

- `SensorResult`
- `Observation`
- `AgentMetadata`
- `EvidenceRecord`
- `PayloadValidationResult`

These types should live in `libs/common`, while sensor crates implement translation from their native runtime data into the shared model.

This boundaries keeps the backend logic generic and prevents brittle coupling between sensor implementations and orchestration code.

---

## 6. Practical rules for implementation

1. Add no crate-to-crate dependency that bypasses `libs/common` for business-domain types.
2. Shared enums, scoring metadata, and schema definitions belong in `libs/common`.
3. Keep all database access outside the sensor crates.
4. Do not let the backend depend on sensor-specific runtime crates other than through interfaces.
5. Only allow the backend to talk to the sensors through the same ingestion contract.
6. Keep web frontend concerns out of the sensor implementation boundary.

---

## 7. Example dependency diagram

```text
libs/common
  ├─> apps/backend
  ├─> sensors/network
  ├─> sensors/device
  ├─> sensors/remote
  └─> sensors/discovery

apps/backend
  ├─> database drivers
  ├─> secret manager client
  ├─> HTTP framework
  └─> libs/common

sensors/*
  ├─> libs/common
  ├─> system/network libraries
  └─> optional parse/validation helpers
```

This is the intended architecture for the first implementation wave.

---
