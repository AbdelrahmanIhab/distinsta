#!/bin/bash

# Phase 2 Quick Test Script
# This script helps test the basic Phase 2 functionality

echo "=== Phase 2 Test Setup ==="
echo ""

# Build the project
echo "1. Building project..."
cargo build --bin server --bin client 2>&1 | grep -E "Finished|error"

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✓ Build successful"
echo ""

# Create test images if they don't exist
if [ ! -d "test_images" ]; then
    echo "2. Creating test images..."
    mkdir -p test_images
    python3 -c "
from PIL import Image
img1 = Image.new('RGB', (400, 300), color=(73, 109, 137))
img1.save('test_images/test1.png')
img2 = Image.new('RGB', (400, 300), color=(200, 100, 50))
img2.save('test_images/test2.png')
img3 = Image.new('RGB', (400, 300), color=(50, 200, 100))
img3.save('test_images/test3.png')
print('✓ Test images created')
"
fi

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Now you can test Phase 2 features:"
echo ""
echo "Terminal 1 - Start Server:"
echo "  cargo run --bin server 1"
echo ""
echo "Terminal 2 - Start Alice's Client:"
echo "  cargo run --bin client alice 9001"
echo ""
echo "Terminal 3 - Start Bob's Client:"
echo "  cargo run --bin client bob 9002"
echo ""
echo "=== Example Test Workflow ==="
echo ""
echo "1. In Alice's terminal:"
echo "   > peers                                      # Check if Bob is online"
echo "   > share test_images/test1.png img001 bob 5  # Share image with Bob (5 views)"
echo ""
echo "2. In Bob's terminal:"
echo "   > peers                                      # See Alice is online"
echo "   > request alice img001 127.0.0.1:9001       # Request image from Alice"
echo "   > view img001                                # View image (decrements to 4)"
echo "   > view img001                                # View again (decrements to 3)"
echo "   > view img001                                # Continue..."
echo "   > view img001                                # (2 remaining)"
echo "   > view img001                                # (1 remaining)"
echo "   > view img001                                # (0 remaining - access denied!)"
echo ""
echo "3. Test P2P features:"
echo "   > my_images                                  # List owned images"
echo "   > received                                   # List received images"
echo ""
echo "For distributed testing across 3 machines, update config.toml with machine IPs"
echo ""
