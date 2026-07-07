#!/bin/bash
# Run script for colour_track binary
# Main color tracking program for Paintball Sentry Turret

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Check if binary exists
if [ -f "target/release/colour_track" ]; then
    BINARY="target/release/colour_track"
elif [ -f "target/debug/colour_track" ]; then
    BINARY="target/debug/colour_track"
else
    echo "Binary not found. Building first..."
    ./scripts/build.sh
    BINARY="target/debug/colour_track"
fi

echo "=========================================="
echo "Paintball Sentry Turret - Colour Track"
echo "=========================================="
echo ""
echo "Starting color tracking..."
echo "Press ESC in the video window to exit"
echo ""

# Run the binary
exec "$BINARY"