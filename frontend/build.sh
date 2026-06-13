#!/bin/bash

# Build the frontend with trunk (production release)

set -e

cd "$(dirname "$0")"

if ! command -v trunk &> /dev/null; then
    echo "Error: trunk is not installed."
    echo "Install with: cargo install --locked trunk"
    exit 1
fi

echo "Building frontend (release)..."
trunk build --release

echo "Frontend build complete!"
