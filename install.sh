#!/bin/bash

# Installation script for clipq
set -e

echo "Installing clipq..."

# Build the project
./build.sh

# Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy binary
cp target/release/clipq "$INSTALL_DIR/"

# Make it executable
chmod +x "$INSTALL_DIR/clipq"

# Create config directory
CONFIG_DIR="$HOME/.clipq"
mkdir -p "$CONFIG_DIR"

# Copy default config if it doesn't exist
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cp .clipq.toml "$CONFIG_DIR/config.toml"
fi

echo "Installation complete!"
echo ""
echo "Binary installed to: $INSTALL_DIR/clipq"
echo "Config directory: $CONFIG_DIR"
echo ""
echo "Add $INSTALL_DIR to your PATH if it's not already there:"
echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "Usage:"
echo "  clipq --help"
echo "  clipq daemon"
echo "  clipq pick"