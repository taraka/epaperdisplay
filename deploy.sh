#!/usr/bin/env bash
set -euo pipefail

PI_HOST="${PI_HOST:-calender}"
PI_USER="${PI_USER:-tom}"
PI_BINARY_PATH="${PI_BINARY_PATH:-/home/${PI_USER}/epaperdisplay}"
TARGET="aarch64-unknown-linux-gnu"
BINARY="target/${TARGET}/release/epaperdisplay"

echo "Building for ${TARGET}..."
cargo zigbuild --release --target "${TARGET}"

echo "Deploying to ${PI_USER}@${PI_HOST}:${PI_BINARY_PATH}..."
ssh "${PI_USER}@${PI_HOST}" "sudo systemctl stop epaper"
scp "${BINARY}" "${PI_USER}@${PI_HOST}:${PI_BINARY_PATH}"
ssh "${PI_USER}@${PI_HOST}" "sudo systemctl start epaper"

echo "Done."
