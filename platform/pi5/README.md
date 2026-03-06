Raspberry Pi 5 (aarch64) build helpers
=======================================

This folder contains scripts and helpers to build the UHTCP project for a
64-bit Raspberry Pi 5 (aarch64 / `arm64`). The scripts use Docker with
`--platform=linux/arm64` so you can cross-build on an x86_64 host without
installing cross toolchains locally.

Layout
- `build_all.sh` : orchestrates builds for Rust core, Go server, C++ control, and HAL C code.
- `build_rust.sh` : builds `core` crate for `aarch64-unknown-linux-gnu` inside an arm64 container.
- `build_go.sh`   : builds `server` Go binaries for `linux/arm64` inside an arm64 container.
- `build_cpp.sh`  : builds `control` C++ static library using an arm64 toolchain inside a container.
- `build_hal.sh`  : builds `hal` C code into a static lib for arm64.
- `out/`          : build outputs (created by scripts).

Usage

Prerequisites: Docker installed on the host and ability to pull multi-arch images.

From repository root run:

```bash
cd platform/pi5
./build_all.sh
```

Artifacts will be in `platform/pi5/out/`. You can copy those to a Pi 5 device.

Notes
- These scripts prefer Docker's `--platform=linux/arm64` to run arm64 images on an x86 host. If your Docker does not support emulation, run the scripts on an aarch64 host or use QEMU user emulation.
- The Rust build uses `aarch64-unknown-linux-gnu` target; adjust to `musl` targets if you need fully static binaries.
- The C/C++ builds use aarch64 cross compilers inside the container; the scripts install basic build tools inside the transient container.
