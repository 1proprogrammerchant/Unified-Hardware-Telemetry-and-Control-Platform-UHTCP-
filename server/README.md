# UHTCP Server (scaffold)

This folder contains a minimal scaffold for the Go API/server used to front the core.

Quick start (from repository root):

```bash
cd server
go mod tidy
go run ./cmd/server
```

Endpoints:
- `GET /api/v1/health` — proxies the core `/health` endpoint
- `GET /metrics` — Prometheus metrics endpoint
- `GET /ws` — websocket upgrade (hub broadcasts received messages)

Notes:
- This is a scaffold: the `internal.IPCClient` is a synchronous client today. Async queueing and retries can be added to `internal/ipc_client.go`.
- The scaffold uses `github.com/gorilla/websocket` and `github.com/prometheus/client_golang`.

Packaging & install
 - A simple `install.sh` script is provided to build and copy the server to `/opt/uhtcp` and install a systemd unit: `./install.sh /opt/uhtcp`
 - A `Dockerfile` is available under `docker/` for container builds.
 - Default configuration lives at `config/default.yaml`.
