#!/bin/bash

# Build script for the frontend

set -e

echo "Building frontend with wasm-pack..."

# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the frontend
cd "$(dirname "$0")"
wasm-pack build --dev --target web --out-dir dist

# Copy index.html to dist
cp index.html dist/

echo "Frontend build complete!"