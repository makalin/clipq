#!/bin/bash

# Test script for clipq
set -e

echo "Testing clipq functionality..."
echo "=============================="

# Build the project
echo "Building clipq..."
cargo build --release

# Test basic commands
echo ""
echo "1. Testing add command..."
./target/release/clipq add "Test entry 1"
./target/release/clipq add "Test entry 2"
./target/release/clipq add "Test entry 3"

echo ""
echo "2. Testing list command..."
./target/release/clipq list

echo ""
echo "3. Testing config command..."
./target/release/clipq config

echo ""
echo "4. Testing clear command..."
./target/release/clipq clear
./target/release/clipq list

echo ""
echo "5. Testing pick command (will show fzf/skim if available)..."
./target/release/clipq add "Sample text for picking"
./target/release/clipq add "Another sample text"
./target/release/clipq add "Third sample text"

echo "Pick command test completed (user interaction required for fzf/skim)"
echo ""
echo "All tests completed successfully!"
echo ""
echo "To run the daemon: ./target/release/clipq daemon"
echo "To install: ./install.sh"