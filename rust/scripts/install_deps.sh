#!/bin/bash
# Installation script for Paintball Sentry Turret dependencies
# This script installs Rust toolchain and system dependencies for Raspberry Pi

set -e

echo "=========================================="
echo "Paintball Sentry Turret - Dependency Installation"
echo "=========================================="
echo ""

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$NAME
else
    OS="Unknown"
fi

echo "Detected OS: $OS"
echo ""

# Function to install Rust
install_rust() {
    echo "Checking Rust installation..."
    if command -v cargo &> /dev/null; then
        echo "Rust is already installed: $(cargo --version)"
    else
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo "Rust installed: $(cargo --version)"
    fi
}

# Function to install system dependencies on Debian/Ubuntu/Raspberry Pi OS
install_debian_deps() {
    echo "Installing system dependencies for Debian/Ubuntu/Raspberry Pi OS..."
    
    sudo apt-get update
    
    # OpenCV dependencies
    sudo apt-get install -y \
        libopencv-dev \
        clang \
        libclang-dev \
        pkg-config \
        cmake \
        build-essential
    
    # Video capture dependencies
    sudo apt-get install -y \
        libv4l-dev \
        v4l-utils \
        libgstreamer1.0-dev \
        libgstreamer-plugins-base1.0-dev
    
    # GUI dependencies (for highgui)
    sudo apt-get install -y \
        libgtk-3-dev \
        libgtk2.0-dev
    
    # Serial port dependencies
    sudo apt-get install -y \
        libudev-dev
    
    # Additional utilities
    sudo apt-get install -y \
        git \
        curl \
        wget
    
    echo "System dependencies installed successfully!"
}

# Function to install system dependencies on Fedora
install_fedora_deps() {
    echo "Installing system dependencies for Fedora..."
    
    sudo dnf install -y \
        opencv-devel \
        clang-devel \
        pkg-config \
        cmake \
        gcc-c++ \
        libv4l-devel \
        gtk3-devel \
        systemd-devel \
        git \
        curl \
        wget
    
    echo "System dependencies installed successfully!"
}

# Function to install system dependencies on Arch Linux
install_arch_deps() {
    echo "Installing system dependencies for Arch Linux..."
    
    sudo pacman -Syu --noconfirm \
        opencv \
        clang \
        pkg-config \
        cmake \
        v4l-utils \
        gtk3 \
        systemd \
        git \
        curl \
        wget
    
    echo "System dependencies installed successfully!"
}

# Main installation flow
echo "Step 1: Installing Rust toolchain"
echo "----------------------------------"
install_rust
echo ""

echo "Step 2: Installing system dependencies"
echo "---------------------------------------"

case "$OS" in
    *"Ubuntu"*|*"Debian"*|*"Raspbian"*|*"Raspberry Pi OS"*)
        install_debian_deps
        ;;
    *"Fedora"*)
        install_fedora_deps
        ;;
    *"Arch"*)
        install_arch_deps
        ;;
    *)
        echo "Warning: Unknown OS detected. Attempting Debian-based installation..."
        if command -v apt-get &> /dev/null; then
            install_debian_deps
        else
            echo "Error: Could not determine package manager."
            echo "Please install the following manually:"
            echo "  - OpenCV development libraries"
            echo "  - Clang and libclang"
            echo "  - pkg-config"
            echo "  - cmake"
            echo "  - GTK3 development libraries"
            echo "  - libudev development libraries"
            exit 1
        fi
        ;;
esac

echo ""
echo "Step 3: Verifying installation"
echo "-------------------------------"

# Verify Rust
if command -v cargo &> /dev/null; then
    echo "✓ Rust: $(cargo --version)"
else
    echo "✗ Rust: Not found"
fi

# Verify pkg-config
if command -v pkg-config &> /dev/null; then
    echo "✓ pkg-config: $(pkg-config --version)"
else
    echo "✗ pkg-config: Not found"
fi

# Verify OpenCV
if pkg-config --exists opencv4 2>/dev/null; then
    echo "✓ OpenCV: $(pkg-config --modversion opencv4)"
elif pkg-config --exists opencv 2>/dev/null; then
    echo "✓ OpenCV: $(pkg-config --modversion opencv)"
else
    echo "✗ OpenCV: Not found via pkg-config"
fi

echo ""
echo "=========================================="
echo "Installation complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Build the project: ./scripts/build.sh"
echo "  2. Run calibration:   ./scripts/run_calibrate.sh"
echo "  3. Run tracking:      ./scripts/run_colour_track.sh"
echo ""
echo "For Raspberry Pi deployment:"
echo "  - Build with release mode: ./scripts/build.sh --release"
echo "  - Binaries will be in target/release/"