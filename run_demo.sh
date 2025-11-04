#!/bin/bash

echo "=== Distributed Image Storage System Demo ==="
echo ""
echo "This script will start 3 server nodes and demonstrate the system."
echo ""

# Build the project
echo "Building the project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Build successful!"
echo ""
echo "To run the system:"
echo ""
echo "1. Open 3 terminals and run:"
echo "   Terminal 1: cargo run --bin server 1"
echo "   Terminal 2: cargo run --bin server 2"
echo "   Terminal 3: cargo run --bin server 3"
echo ""
echo "2. Wait 5 seconds for leader election to complete"
echo ""
echo "3. In a 4th terminal, run the client:"
echo "   cargo run --bin client alice 127.0.0.1:8003"
echo ""
echo "4. Use the interactive menu to upload/download images"
echo ""
echo "Note: Node 3 (highest ID) will become the leader"
