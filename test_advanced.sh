#!/bin/bash

# Advanced test script for clipq with all new features
set -e

echo "Testing clipq advanced functionality..."
echo "======================================"

# Build the project
echo "Building clipq..."
cargo build --release

# Test basic functionality
echo ""
echo "1. Testing basic commands..."
./target/release/clipq add "Hello, World! This is a test entry."
./target/release/clipq add "Another test entry with some content."
./target/release/clipq add "https://example.com - This contains a URL"
./target/release/clipq add "Contact me at user@example.com for more info"

echo ""
echo "2. Testing search functionality..."
./target/release/clipq search "test"
./target/release/clipq search "URL"

echo ""
echo "3. Testing statistics..."
./target/release/clipq stats

echo ""
echo "4. Testing file functionality..."
echo "This is a test file content" > test_file.txt
./target/release/clipq file test_file.txt
rm test_file.txt

echo ""
echo "5. Testing tags..."
./target/release/clipq tag 1 "important"
./target/release/clipq tag 2 "work"
./target/release/clipq tags

echo ""
echo "6. Testing export/import..."
./target/release/clipq export --format json --output test_export.json
./target/release/clipq clear
./target/release/clipq list
./target/release/clipq import --format json --input test_export.json
./target/release/clipq list
rm test_export.json

echo ""
echo "7. Testing backup/restore..."
./target/release/clipq backup --output test_backup.db
./target/release/clipq clear
./target/release/clipq list
./target/release/clipq restore --input test_backup.db
./target/release/clipq list
rm test_backup.db

echo ""
echo "8. Testing plugins..."
./target/release/clipq plugins

echo ""
echo "9. Testing built-in utilities..."
./target/release/clipq extract-urls "Visit https://github.com and https://stackoverflow.com for help"
./target/release/clipq generate-password --length 20
./target/release/clipq hash "Hello, World!" --algorithm sha256

echo ""
echo "10. Testing web interface (will start server - press Ctrl+C to stop)..."
echo "Starting web server on http://localhost:8080"
echo "Open your browser and visit http://localhost:8080"
echo "Press Ctrl+C to stop the web server"
./target/release/clipq web --port 8080

echo ""
echo "All advanced tests completed successfully!"
echo ""
echo "Available commands:"
echo "  clipq --help                    # Show all commands"
echo "  clipq daemon                    # Run clipboard daemon"
echo "  clipq web                       # Start web interface"
echo "  clipq search <query>            # Search clipboard history"
echo "  clipq stats                     # Show statistics"
echo "  clipq export --format json      # Export clipboard history"
echo "  clipq import --format json      # Import clipboard history"
echo "  clipq tags                      # Show clips with tags"
echo "  clipq tag <clip> <tag>          # Add tag to clip"
echo "  clipq plugins                   # List available plugins"
echo "  clipq generate-password         # Generate random password"
echo "  clipq hash <text>               # Calculate hash"
echo "  clipq extract-urls <text>       # Extract URLs from text"