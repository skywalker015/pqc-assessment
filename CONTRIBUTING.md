# Contributing to the PQC Readiness Platform

Welcome! This document outlines how to set up your local development environment for the PQC Readiness Platform monorepo.

## 1. Prerequisites
- **Rust Toolchain:** Install via `rustup` (Ensure you have the latest stable version).
- **Node.js:** v18 or higher for the web dashboard.
- **SQLite:** Pre-installed on most Unix systems.

## 2. Local Setup
1. Clone the repository.
2. Initialize the SQLite database:
   ```bash
   mkdir -p data
   sqlite3 data/pqc_assessment.db < scripts/schema.sql
   ```
3. Build the shared libraries and backend:
   ```bash
   cargo build
   ```

## 3. Running the Backend
To start the backend with Tokio and watch for changes:
```bash
cargo run --bin backend
```

## 4. Code Standards
- **Formatting:** Run `cargo fmt` before every commit.
- **Linting:** We strictly enforce clippy lints. Run `cargo clippy -- -D warnings` to ensure your code passes.
- **Testing:** Run `cargo test` to execute all unit and integration tests.

## 5. mTLS Local Development
For local testing of sensors, use the `scripts/generate_certs.sh` script to generate local CA and client certificates for the sensors. Ensure these are loaded in your local environment variables.

