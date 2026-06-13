#!/bin/bash

# Build the entire pi-brain workspace (frontend + backend)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Building pi-brain workspace ==="

# Build frontend first (backend serves the static output)
echo "[1/2] Building frontend..."
cd frontend
./build.sh
cd ..

# Build backend (release binary)
echo "[2/2] Building backend..."
cargo build --release

echo "=== Build complete ==="
echo "Restart the service with: systemctl --user restart pi-brain"
