#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=${1:-$(pwd)/out}
ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)

echo "Running Rust static-musl build inside arm64 container..."

docker run --rm --platform linux/arm64 \
  -v "$ROOT_DIR":/work -w /work/core \
  rust:1.76-slim-bullseye bash -lc '
    set -e
    apt-get update && apt-get install -y --no-install-recommends build-essential musl-tools ca-certificates curl && rm -rf /var/lib/apt/lists/*
    rustup default stable || true
    rustup target add aarch64-unknown-linux-musl || true
    export CC=musl-gcc
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
    cargo build --release --target aarch64-unknown-linux-musl
    '

mkdir -p "$OUT_DIR"
cp -v "$ROOT_DIR/core/target/aarch64-unknown-linux-musl/release/uhtcp-core" "$OUT_DIR/" 2>/dev/null || true
echo "Rust static-musl build finished (artifact copied if available)."
