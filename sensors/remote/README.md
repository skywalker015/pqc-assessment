# Remote Sensor Agent

Credentialed remote assessment agent for device PQC inspection.

This agent connects to target devices via SSH using user-supplied credentials (password or private key) and executes commands to gather local PQC-related configuration, such as the OpenSSL version and OpenSSH configurations. The collected data is then sent back to the backend service.

## Usage

```bash
cargo run -p remote-agent -- --host <target_ip> --user <username> [OPTIONS]
```

### Options
- `--host`: Target host to connect to (required).
- `--port`: SSH port (default: 22).
- `--user`: SSH username (required).
- `--password`: SSH password (optional).
- `--private-key` / `-k`: Path to private SSH key (optional).
- `--backend-url`: Backend gRPC URL (default: http://127.0.0.1:50051).
- `--device-id`: Device ID for this host (default: unknown).

If neither `--password` nor `--private-key` is provided, the agent will attempt to use the SSH agent for authentication.
