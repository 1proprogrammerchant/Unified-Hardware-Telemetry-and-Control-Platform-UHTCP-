#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)
OUT_DIR="$ROOT_DIR/platform/pi5/out"
mkdir -p "$OUT_DIR"

echo "Building Rust core..."
"$(dirname "$0")/build_rust.sh" "$OUT_DIR"

echo "Building Go server..."
"$(dirname "$0")/build_go.sh" "$OUT_DIR"

echo "Building C++ control..."
"$(dirname "$0")/build_cpp.sh" "$OUT_DIR"

echo "Building HAL..."
"$(dirname "$0")/build_hal.sh" "$OUT_DIR"

echo "All done. Artifacts in: $OUT_DIR"
