#!/bin/bash

# Build script for clipq
set -e

echo "Building clipq..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build in release mode
echo "Building release version..."
cargo build --release

echo "Build complete!"
echo "Binary location: target/release/clipq"

# Check if fzf is installed
if ! command -v fzf &> /dev/null; then
    echo "Warning: fzf is not installed. Install it for the best experience:"
    echo "  macOS: brew install fzf"
    echo "  Ubuntu/Debian: apt install fzf"
    echo "  Or visit: https://github.com/junegunn/fzf"
fi

echo ""
echo "Usage:"
echo "  ./target/release/clipq --help"
echo "  ./target/release/clipq daemon"
echo "  ./target/release/clipq pick"