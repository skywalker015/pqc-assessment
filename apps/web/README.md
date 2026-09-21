# Web Frontend

This folder contains the web UI for the PQC readiness application.

The dashboard is served by the Rust backend at `http://127.0.0.1:3000/`. The
backend embeds `index.html` into its binary and supplies live data through
`GET /api/dashboard`; no Python runtime is required.

Planned responsibilities:
- Dashboard
- Asset and service inventory
- Sensor health
- CSV upload for device-agent results
- Reports and scorecards
- User and audit management
