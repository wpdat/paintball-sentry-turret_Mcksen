#!/bin/bash
# Test script for Paintball Sentry Turret (Rust)
# Runs unit tests for all modules

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Paintball Sentry Turret - Test Suite"
echo "=========================================="

cd "$PROJECT_DIR"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please run ./scripts/install_deps.sh first"
    exit 1
fi

echo ""
echo "Running unit tests..."
echo ""

# Run tests
cargo test --lib

echo ""
echo "=========================================="
echo "All tests passed!"
echo "=========================================="