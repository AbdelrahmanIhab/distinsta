#!/bin/bash

# Stress Test for Load Balancing
# This script tests that load is distributed across all server nodes

echo "=========================================="
echo "Load Balancing Stress Test"
echo "=========================================="
echo ""

# Configuration
NUM_UPLOADS=30
USERNAME="testuser"
TEST_IMAGE="test_image.png"

# Check if test image exists
if [ ! -f "$TEST_IMAGE" ]; then
    echo "Error: $TEST_IMAGE not found!"
    echo "Please ensure test_image.png exists in the current directory"
    exit 1
fi

# Create temporary directory for test files
TEST_DIR=$(mktemp -d)
echo "Creating test files in: $TEST_DIR"

# Generate multiple copies with different names
for i in $(seq 1 $NUM_UPLOADS); do
    cp "$TEST_IMAGE" "$TEST_DIR/image_${i}.png"
done

echo "Created $NUM_UPLOADS test images"
echo ""

# Create upload commands file
COMMANDS_FILE=$(mktemp)

echo "Generating upload commands..."
for i in $(seq 1 $NUM_UPLOADS); do
    echo "upload $TEST_DIR/image_${i}.png" >> "$COMMANDS_FILE"
done
echo "quit" >> "$COMMANDS_FILE"

echo "Commands file created with $NUM_UPLOADS uploads"
echo ""

# Run the client with all commands
echo "=========================================="
echo "Starting uploads..."
echo "=========================================="
echo ""

OUTPUT_FILE=$(mktemp)
cargo run --bin client "$USERNAME" < "$COMMANDS_FILE" 2>&1 | tee "$OUTPUT_FILE"

echo ""
echo "=========================================="
echo "Analyzing Results"
echo "=========================================="
echo ""

# Count how many requests each server processed
NODE1_COUNT=$(grep -c "Server 1 processed request" "$OUTPUT_FILE" || echo "0")
NODE2_COUNT=$(grep -c "Server 2 processed request" "$OUTPUT_FILE" || echo "0")
NODE3_COUNT=$(grep -c "Server 3 processed request" "$OUTPUT_FILE" || echo "0")

# Also check for declined messages
NODE1_DECLINED=$(grep -c "Server 1 declined:" "$OUTPUT_FILE" || echo "0")
NODE2_DECLINED=$(grep -c "Server 2 declined:" "$OUTPUT_FILE" || echo "0")
NODE3_DECLINED=$(grep -c "Server 3 declined:" "$OUTPUT_FILE" || echo "0")

echo "Upload Distribution:"
echo "-------------------"
echo "Node 1: $NODE1_COUNT processed, $NODE1_DECLINED declined"
echo "Node 2: $NODE2_COUNT processed, $NODE2_DECLINED declined"
echo "Node 3: $NODE3_COUNT processed, $NODE3_DECLINED declined"
echo ""

TOTAL_PROCESSED=$((NODE1_COUNT + NODE2_COUNT + NODE3_COUNT))
echo "Total processed: $TOTAL_PROCESSED out of $NUM_UPLOADS requests"
echo ""

# Calculate percentages
if [ $TOTAL_PROCESSED -gt 0 ]; then
    NODE1_PCT=$(awk "BEGIN {printf \"%.1f\", ($NODE1_COUNT / $TOTAL_PROCESSED) * 100}")
    NODE2_PCT=$(awk "BEGIN {printf \"%.1f\", ($NODE2_COUNT / $TOTAL_PROCESSED) * 100}")
    NODE3_PCT=$(awk "BEGIN {printf \"%.1f\", ($NODE3_COUNT / $TOTAL_PROCESSED) * 100}")

    echo "Load Distribution (%):"
    echo "---------------------"
    echo "Node 1: ${NODE1_PCT}%"
    echo "Node 2: ${NODE2_PCT}%"
    echo "Node 3: ${NODE3_PCT}%"
    echo ""
fi

# Determine if load balancing is working
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo ""

NODES_USED=0
[ $NODE1_COUNT -gt 0 ] && NODES_USED=$((NODES_USED + 1))
[ $NODE2_COUNT -gt 0 ] && NODES_USED=$((NODES_USED + 1))
[ $NODE3_COUNT -gt 0 ] && NODES_USED=$((NODES_USED + 1))

if [ $NODES_USED -eq 3 ]; then
    echo "✓ PASS: All 3 nodes are processing requests"
    echo "✓ Load balancing is working!"

    # Check if distribution is reasonably balanced
    # (Allow for some variance - each node should have at least 20% of work)
    MIN_EXPECTED=$((TOTAL_PROCESSED / 5))  # 20% threshold

    BALANCED=true
    [ $NODE1_COUNT -lt $MIN_EXPECTED ] && BALANCED=false
    [ $NODE2_COUNT -lt $MIN_EXPECTED ] && BALANCED=false
    [ $NODE3_COUNT -lt $MIN_EXPECTED ] && BALANCED=false

    if [ "$BALANCED" = true ]; then
        echo "✓ Distribution is well-balanced"
    else
        echo "⚠ Distribution is uneven (expected >20% per node)"
    fi
elif [ $NODES_USED -eq 1 ]; then
    echo "✗ FAIL: Only 1 node is processing requests"
    echo "✗ Load balancing is NOT working - all requests going to one node!"
    ACTIVE_NODE=""
    [ $NODE1_COUNT -gt 0 ] && ACTIVE_NODE="Node 1"
    [ $NODE2_COUNT -gt 0 ] && ACTIVE_NODE="Node 2"
    [ $NODE3_COUNT -gt 0 ] && ACTIVE_NODE="Node 3"
    echo "  Active node: $ACTIVE_NODE"
else
    echo "⚠ WARNING: Only $NODES_USED nodes are processing requests"
    echo "  Check if all servers are running"
fi

echo ""

# Cleanup
echo "Cleaning up temporary files..."
rm -rf "$TEST_DIR"
rm -f "$COMMANDS_FILE"
rm -f "$OUTPUT_FILE"

echo "Done!"
