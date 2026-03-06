#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=${1:-$(pwd)/out}
ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)

echo "Running Go build inside arm64 container..."

docker run --rm --platform linux/arm64 \
  -v "$ROOT_DIR":/work -w /work/server \
  golang:1.20 bash -lc '
    set -e
    mkdir -p /out
    CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build -ldflags "-s -w" -o /out/uhtcp-server ./cmd/server
    '

mkdir -p "$OUT_DIR"
docker run --rm --platform linux/arm64 -v "$ROOT_DIR":/work -v "$OUT_DIR":/out -w /work/server alpine:3.18 sh -c 'cp /out/uhtcp-server /out/ || true'
echo "Go build finished (artifact copied if available)."
