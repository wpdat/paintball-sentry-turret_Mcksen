#!/bin/bash
# Run script for calibrate_range binary
# HSV color calibration tool for Paintball Sentry Turret

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Check if binary exists
if [ -f "target/release/calibrate_range" ]; then
    BINARY="target/release/calibrate_range"
elif [ -f "target/debug/calibrate_range" ]; then
    BINARY="target/debug/calibrate_range"
else
    echo "Binary not found. Building first..."
    ./scripts/build.sh
    BINARY="target/debug/calibrate_range"
fi

echo "=========================================="
echo "Paintball Sentry Turret - HSV Calibration"
echo "=========================================="
echo ""
echo "Starting HSV calibration tool..."
echo "Adjust trackbars to find optimal color range"
echo "Press ESC in the video window to exit"
echo ""

# Run the binary
exec "$BINARY"