#!/bin/bash

# Build script for the entire workspace

set -e

echo "Building Knowledge Base workspace..."

# Build frontend first
echo "Building frontend..."
cd frontend
./build.sh
cd ..

# Build backend
echo "Building backend..."
cargo build --release

echo "Build complete!"
echo "Run with: cargo run --release --bin knowledge-base"