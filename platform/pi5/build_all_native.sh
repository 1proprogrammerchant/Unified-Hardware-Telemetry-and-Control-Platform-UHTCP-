#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)
OUT_DIR="$ROOT_DIR/platform/pi5/out_native"
mkdir -p "$OUT_DIR"

echo "== UHTCP Native Build (host) =="
echo "Output dir: $OUT_DIR"

echo "\n-- Building Rust core (release) --"
if command -v cargo >/dev/null 2>&1; then
  pushd "$ROOT_DIR/core" >/dev/null
  cargo clean || true
  # If sccache isn't installed locally, unset RUSTC_WRAPPER so cargo won't try to use it
  if ! command -v sccache >/dev/null 2>&1; then
    echo "sccache not found; unsetting RUSTC_WRAPPER to avoid rustc-wrapper errors"
    export RUSTC_WRAPPER=""
  fi
  if rustup target list --installed | grep -q "aarch64-unknown-linux-musl"; then
    echo "Building static musl target aarch64-unknown-linux-musl (host may need cross toolchain)"
    cargo build --release --target aarch64-unknown-linux-musl || cargo build --release
  else
    echo "musl target not installed; building native release"
    cargo build --release
  fi
  popd >/dev/null
  # try to copy artifact if present
  if [ -f "$ROOT_DIR/core/target/aarch64-unknown-linux-musl/release/uhtcp-core" ]; then
    cp "$ROOT_DIR/core/target/aarch64-unknown-linux-musl/release/uhtcp-core" "$OUT_DIR/" || true
  elif [ -f "$ROOT_DIR/core/target/release/uhtcp-core" ]; then
    cp "$ROOT_DIR/core/target/release/uhtcp-core" "$OUT_DIR/" || true
  fi
else
  echo "cargo not found; skipping Rust build"
fi

echo "\n-- Building Go server --"
echo "\n-- Building Go server --"
# Ensure we have a usable Go toolchain available locally.
GO_TOOLCHAIN="$OUT_DIR/go"
GO_BIN="$GO_TOOLCHAIN/bin/go"
if [ -x "$GO_BIN" ]; then
  echo "Using local downloaded Go at $GO_BIN"
  PATH="$GO_TOOLCHAIN/bin:$PATH"
elif command -v go >/dev/null 2>&1; then
  echo "Using host Go: $(go version)"
else
  echo "No suitable Go found; downloading portable Go 1.20.10 into $GO_TOOLCHAIN"
  mkdir -p "$OUT_DIR"
  curl -fsSL -o "$OUT_DIR/go1.20.10.linux-amd64.tar.gz" https://go.dev/dl/go1.20.10.linux-amd64.tar.gz || { echo "failed to download Go"; }
  rm -rf "$GO_TOOLCHAIN" && mkdir -p "$OUT_DIR/tmpgo" && tar -C "$OUT_DIR/tmpgo" -xzf "$OUT_DIR/go1.20.10.linux-amd64.tar.gz" || true
  # move into canonical location
  mv "$OUT_DIR/tmpgo/go" "$GO_TOOLCHAIN" 2>/dev/null || true
  PATH="$GO_TOOLCHAIN/bin:$PATH"
fi

if command -v go >/dev/null 2>&1; then
  pushd "$ROOT_DIR/server" >/dev/null
  env CGO_ENABLED=0 GOOS=linux GOARCH=arm64 "$GO_BIN" build -ldflags "-s -w" -o "$OUT_DIR/uhtcp-server" ./cmd/server || env GOOS=linux GOARCH=arm64 "$GO_BIN" build -o "$OUT_DIR/uhtcp-server" ./cmd/server
  popd >/dev/null
else
  echo "go still not available; skipping Go build"
fi

echo "\n-- Building C++ control library --"
if command -v make >/dev/null 2>&1; then
  pushd "$ROOT_DIR/control" >/dev/null
  make || true
  popd >/dev/null
  if [ -f "$ROOT_DIR/control/libcontrol.a" ]; then
    cp "$ROOT_DIR/control/libcontrol.a" "$OUT_DIR/" || true
  fi
else
  echo "make not found; skipping C++ build"
fi

echo "\n-- Building HAL (C) --"
if command -v make >/dev/null 2>&1; then
  pushd "$ROOT_DIR/hal" >/dev/null
  make || true
  popd >/dev/null
  if [ -f "$ROOT_DIR/hal/libuhtcp_hal.a" ]; then
    cp "$ROOT_DIR/hal/libuhtcp_hal.a" "$OUT_DIR/" || true
  fi
else
  echo "make not found; skipping HAL build"
fi

echo "\nNative build finished. Artifacts (if built) are in: $OUT_DIR"
