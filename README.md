# UHTCP — Unified Hardware Telemetry and Control Platform

Repository scaffold with minimal examples across languages:

- hal: C hardware abstraction layer
- control: C++ hardware control engine
- core: Rust core engine (scheduler/polling)
- server: Go network API server
- automation: Ruby automation scripts

Quick start (Linux-like host recommended):


Build & run C++ sensor runner:

```bash
g++ -std=c++17 control/Sensor.cpp hal/hardware.c -Ihal -o control/sensor_app
./control/sensor_app
```

Build and run Rust core (uses `cc` build to compile the C HAL):

```bash
cd core
cargo build --release
./target/release/uhtcp-core
```

Run Go API server (proxies Rust core state):

```bash
go run server/api.go
```

Start a Ruby automation script via the Go API (supervisor will spawn the sandbox runner):

```bash
# start script with id "script1"
curl -X POST -d '{"path":"automation/my_script.rb","id":"script1"}' \
	-H 'Content-Type: application/json' http://localhost:8080/api/v1/scripts/start

# stop script
curl -X POST -d '{"id":"script1"}' -H 'Content-Type: application/json' \
	http://localhost:8080/api/v1/scripts/stop
```

Run Ruby automation script:

```bash
ruby automation/scripts.rb
```

Notes:

- Many examples read Linux `/sys` thermal files; adapt for your platform.
- This scaffold is a starting point — integrate FFI, messaging, and packaging next.

Publishing
- To publish this repository on GitHub: create a new empty repo on GitHub, add it as `origin`, and push the `main` branch. See `CONTRIBUTING.md` for one-line commands.
- A CI workflow is included at `.github/workflows/ci.yml` which builds the Go server, the Rust core (nightly), and the C++ control demo on Linux runners.
