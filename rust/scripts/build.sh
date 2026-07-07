#!/bin/bash
# Build script for Paintball Sentry Turret (Rust)
# This script compiles the Rust project for Raspberry Pi

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Paintball Sentry Turret - Build Script"
echo "=========================================="

cd "$PROJECT_DIR"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please run ./scripts/install_deps.sh first"
    exit 1
fi

echo "Building project..."
echo ""

# Build in debug mode by default
if [ "$1" == "--release" ] || [ "$1" == "-r" ]; then
    echo "Building in RELEASE mode..."
    cargo build --release
    echo ""
    echo "Release build complete!"
    echo "Binaries located in: target/release/"
else
    echo "Building in DEBUG mode..."
    echo "(Use --release or -r for optimized release build)"
    cargo build
    echo ""
    echo "Debug build complete!"
    echo "Binaries located in: target/debug/"
fi

echo ""
echo "Available binaries:"
echo "  - colour_track    (main color tracking program)"
echo "  - calibrate_range (HSV calibration tool)"
echo ""
echo "To run:"
echo "  ./scripts/run_colour_track.sh"
echo "  ./scripts/run_calibrate.sh"