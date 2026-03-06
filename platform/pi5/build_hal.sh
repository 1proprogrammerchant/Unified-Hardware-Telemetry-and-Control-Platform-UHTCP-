#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=${1:-$(pwd)/out}
ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)

echo "Running HAL C build inside arm64 container..."

docker run --rm --platform linux/arm64 \
  -v "$ROOT_DIR":/work -w /work/hal \
  ubuntu:22.04 bash -lc '
    set -e
    apt-get update && apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu make ca-certificates && rm -rf /var/lib/apt/lists/*
    export CC=aarch64-linux-gnu-gcc
    make
    '

mkdir -p "$OUT_DIR"
cp -v "$ROOT_DIR/hal/libuhtcp_hal.a" "$OUT_DIR/" 2>/dev/null || true
echo "HAL build finished (artifact copied if available)."
