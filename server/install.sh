#!/usr/bin/env bash
set -euo pipefail

# Lightweight installer for the UHTCP Go server (server component)
# Usage: sudo ./install.sh /opt/uhtcp

PREFIX=${1:-/opt/uhtcp}
BIN_DIR=${PREFIX}/bin
CONF_DIR=${PREFIX}/etc
SERVICE_NAME=uhtcp-server

echo "Installing to ${PREFIX}"
mkdir -p "${BIN_DIR}" "${CONF_DIR}"

echo "Building Go server..."
cd "$(dirname "$0")"
go build -o "${BIN_DIR}/uhtcp-server" ./cmd/server

echo "Copying default config"
cp -n ./config/default.yaml "${CONF_DIR}/default.yaml" || true

if command -v systemctl >/dev/null 2>&1; then
  echo "Installing systemd unit"
  sudo cp -f ./packaging/uhtcp.service /etc/systemd/system/${SERVICE_NAME}.service
  sudo systemctl daemon-reload
  sudo systemctl enable --now ${SERVICE_NAME}.service || true
  echo "Service ${SERVICE_NAME} enabled"
fi

echo "Install complete. Binary: ${BIN_DIR}/uhtcp-server, config: ${CONF_DIR}/default.yaml"
echo "To run locally: ${BIN_DIR}/uhtcp-server --config ${CONF_DIR}/default.yaml"
