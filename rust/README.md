# Paintball Sentry Turret - Rust Implementation

A Rust port of the Python-based paintball sentry turret system, designed for the Raspberry Pi platform.

## Overview

This project implements an automatic color tracking and targeting system for a paintball turret. It captures video from a camera, detects colored objects using HSV color space, calculates target positions using coordinate transformations, and optionally sends commands to an Arduino for servo control.

## Project Structure

```
rust/
├── Cargo.toml              # Rust package configuration
├── README.md               # This file
├── src/
│   ├── lib.rs              # Library root
│   ├── trig.rs             # Coordinate transformations (equivalent to trig.py)
│   ├── common.rs           # Helper functions and OpenCV utilities (equivalent to common.py)
│   └── bin/
│       ├── colour_track.rs     # Main color tracking program (equivalent to colourTrack.py)
│       └── calibrate_range.rs  # HSV calibration tool (equivalent to calibrateRange.py)
└── scripts/
    ├── build.sh            # Build script
    ├── run_colour_track.sh # Run color tracking program
    ├── run_calibrate.sh    # Run calibration tool
    ├── test.sh             # Run unit tests
    └── install_deps.sh     # Install dependencies
```

## Modules (Participants)

### `trig` - Coordinate Transformations
Equivalent to Python's `trig.py`. Contains functions for:
- Converting pixel coordinates to camera angles (`px_to_angle`)
- Converting polar to cartesian coordinates (`cam_polar_cart_z`, `cam_polar_cart_h`)
- Shifting coordinates from camera to gun reference frame (`shift_cart`)
- Converting cartesian to gun polar coordinates (`gun_cart_to_polar`)
- Converting polar angles to servo signals (`signal_from_polar`)
- Computing velocity and predicting positions (`compute_velocity`, `est_next_pos`)
- Main transformation function (`cam_to_gun`)

### `common` - Helper Functions and Utilities
Equivalent to Python's `common.py`. Contains:
- `Bunch` - Simple data container
- `StatValue` - Smoothed statistical value tracker
- `Timer` - Execution time measurement
- `HsvRange` - HSV color range configuration
- `Preferences` - Application configuration
- `ArduinoConfig` - Arduino serial communication settings
- `draw_str()` - Draw text on images
- `find_largest_contour()` - Find and analyze largest contour
- `clock()` - Get current time

### `colour_track` - Main Color Tracking Program
Equivalent to Python's `colourTrack.py`. Features:
- Real-time video capture
- HSV color space filtering
- Contour detection and centroid calculation
- Coordinate transformation to servo commands
- Optional Arduino serial communication
- Optional target prediction (lead)
- Video display with overlay information

### `calibrate_range` - HSV Calibration Tool
Equivalent to Python's `calibrateRange.py`. Features:
- Interactive trackbars for HSV range adjustment
- Real-time color filtering preview
- Contour detection visualization
- Outputs final HSV values for configuration

## Prerequisites

### System Dependencies

For Raspberry Pi OS / Debian / Ubuntu:

```bash
sudo apt-get update
sudo apt-get install -y \
    libopencv-dev \
    clang \
    libclang-dev \
    pkg-config \
    cmake \
    build-essential \
    libv4l-dev \
    libgtk-3-dev \
    libudev-dev
```

### Rust Toolchain

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Or use the provided installation script:

```bash
./scripts/install_deps.sh
```

## Building

### Debug Build (for development)

```bash
./scripts/build.sh
# or
cargo build
```

### Release Build (for deployment)

```bash
./scripts/build.sh --release
# or
cargo build --release
```

## Running

### Color Tracking Program

```bash
./scripts/run_colour_track.sh
# or
cargo run --bin colour_track
```

### HSV Calibration Tool

```bash
./scripts/run_calibrate.sh
# or
cargo run --bin calibrate_range
```

### Running Tests

```bash
./scripts/test.sh
# or
cargo test
```

## Configuration

### Preferences (in `colour_track.rs`)

| Setting | Default | Description |
|---------|---------|-------------|
| `arduino_enabled` | `false` | Enable Arduino serial communication |
| `screen_enabled` | `true` | Enable video display |
| `target_lead_enabled` | `true` | Enable target prediction |
| `write_video_enabled` | `false` | Enable video recording |

### HSV Color Range (in `common.rs`)

Default values for orange paintball detection:

| Channel | Min | Max |
|---------|-----|-----|
| Hue | 10 | 40 |
| Saturation | 70 | 255 |
| Value | 150 | 255 |

Use the calibration tool to find optimal values for your environment.

### Arduino Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| Port | `/dev/ttyACM0` | Serial port |
| Baud Rate | `115200` | Communication speed |

### Servo Signal Protocol

Commands sent to Arduino:
- Pan: `p<value>\n` (e.g., `p538\n`)
- Tilt: `t<value>\n` (e.g., `t488\n`)

Servo value ranges:
- Pan: 251 - 805 (center: 538)
- Tilt: 211 - 765 (center: 488)

## Cross-Compilation for Raspberry Pi

### ARMv7 (32-bit Raspberry Pi)

```bash
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release --target armv7-unknown-linux-gnueabihf
```

### ARM64 (64-bit Raspberry Pi)

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

## Hardware Requirements

- Raspberry Pi (any model with USB)
- USB Webcam (640x480 recommended)
- Arduino (for servo control, optional)
- Pan/Tilt servo mechanism

## Camera Specifications

The coordinate transformations assume:
- Resolution: 640 x 480
- Field of View: 60° (horizontal)
- Aspect Ratio: 4:3
- X-FOV: 48°
- Y-FOV: 36°

## Differences from Python Version

1. **Type Safety**: Rust provides compile-time type checking
2. **Error Handling**: Uses `Result` types instead of exceptions
3. **Memory Safety**: No garbage collection, deterministic resource management
4. **Performance**: Generally faster execution, especially with release builds
5. **Concurrency**: Safe concurrent programming with ownership model
6. **Dependencies**: Uses crates.io packages instead of pip

## License

This project is a port of the original Python implementation for educational purposes.

## Contributing

1. Run tests before submitting changes: `./scripts/test.sh`
2. Ensure code compiles without warnings: `cargo build`
3. Follow Rust coding conventions (use `cargo fmt`)

## Troubleshooting

### OpenCV Not Found

Ensure OpenCV development libraries are installed:
```bash
pkg-config --modversion opencv4
```

### Camera Not Detected

Check video devices:
```bash
ls -la /dev/video*
```

Try changing the device index in the code (0, 1, 2, etc.)

### Serial Port Permission Denied

Add user to dialout group:
```bash
sudo usermod -a -G dialout $USER
```

Then log out and back in.