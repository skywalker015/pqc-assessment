# Web Frontend

This folder contains the web UI for the PQC readiness application.

The dashboard is served by the Rust backend at `http://<host-ip>:3000/` (for
example, `http://10.10.10.101:3000/` on the development host). The backend
listens on all interfaces, embeds `index.html` into its binary, and supplies
live data through `GET /api/dashboard`; no Python runtime is required.

Planned responsibilities:
- Dashboard
- Asset and service inventory
- Sensor health
- CSV upload for device-agent results
- Reports and scorecards
- User and audit management
