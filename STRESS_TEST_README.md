# Load Balancing Stress Test

## Overview

These stress test scripts verify that the load balancing system correctly distributes work across all server nodes.

## Available Tests

### 1. Python Script (Recommended)
```bash
python3 stress_test.py
```

**Features:**
- Clean, colorized output
- Percentage distribution with visual bars
- Detailed analysis of load distribution
- Saves full log to `stress_test_output.log`
- Clear pass/fail criteria

### 2. Bash Script (Alternative)
```bash
./stress_test.sh
```

**Features:**
- No Python dependency
- Same testing logic
- Simpler output format

## How It Works

1. **Creates test files**: Generates 30 copies of `test_image.png` with different filenames
   - `image_1.png`, `image_2.png`, ..., `image_30.png`

2. **Uploads all files**: Sends all 30 images through the client

3. **Analyzes distribution**: Counts how many requests each server processed

4. **Evaluates results**: Determines if load balancing is working correctly

## What to Expect

### Good Load Balancing (All 3 nodes running)

```
Upload Distribution:
--------------------------------------------------
Node 1: 11 processed, 19 declined
Node 2: 9 processed, 21 declined
Node 3: 10 processed, 20 declined

Total processed: 30 out of 30 requests

Load Distribution (%):
--------------------------------------------------
Node 1:  36.7%  ██████████████████
Node 2:  30.0%  ███████████████
Node 3:  33.3%  ████████████████

✓ PASS: All 3 nodes are processing requests
✓ Load balancing is working!
✓ Distribution is well-balanced (>20% per node)
```

### Bad Load Balancing (Only 1 node)

```
Upload Distribution:
--------------------------------------------------
Node 1: 0 processed, 30 declined
Node 2: 0 processed, 30 declined
Node 3: 30 processed, 0 declined

Total processed: 30 out of 30 requests

Load Distribution (%):
--------------------------------------------------
Node 1:   0.0%
Node 2:   0.0%
Node 3: 100.0%  ██████████████████████████████████████████████████

✗ FAIL: Only 1 node is processing requests
✗ Load balancing is NOT working!
  All requests going to: Node 3
```

## Prerequisites

1. **All 3 servers must be running**:
   ```bash
   # Terminal 1
   cargo run --bin server 1

   # Terminal 2
   cargo run --bin server 2

   # Terminal 3
   cargo run --bin server 3
   ```

2. **Wait for leader election** (5-8 seconds)

3. **Test image must exist**:
   - Ensure `test_image.png` exists in the project directory

## Running the Test

```bash
# Python version (recommended)
python3 stress_test.py

# Bash version
./stress_test.sh
```

## Interpreting Results

### Success Criteria

**✓ PASS** - All conditions met:
- All 3 nodes process at least some requests
- Each node handles >20% of total load
- No node handles >40% (optional - excellent balance)

**⚠ WARNING** - Partial success:
- Only 2 nodes processing (one might be down)
- Uneven distribution (hash variance, acceptable)

**✗ FAIL** - Load balancing broken:
- Only 1 node processing all requests
- Load balancing algorithm not working

## Troubleshooting

### Problem: "Only 1 node processing"

**Possible causes:**
1. **Hash function issue** - All hashes map to same node
2. **get_alive_nodes() broken** - Only detecting one node
3. **Network issue** - Can't connect to other nodes

**Debug steps:**
```bash
# Check server logs for "alive nodes" messages
# Should see: "alive nodes: [1, 2, 3]"

# If seeing: "alive nodes: [3]" - health check is failing
```

### Problem: "No servers running"

```bash
# Kill any existing servers
pkill -f "cargo run --bin server"

# Restart all 3
cargo run --bin server 1 &
cargo run --bin server 2 &
cargo run --bin server 3 &

# Wait 8 seconds, then run test
sleep 8
python3 stress_test.py
```

### Problem: "Test image not found"

```bash
# Create a simple test image (requires ImageMagick)
convert -size 100x100 xc:blue test_image.png

# Or use any existing PNG image
cp /path/to/your/image.png test_image.png
```

## Testing Different Scenarios

### Test with 2 Nodes (Simulated Failure)

```bash
# Start only Node 1 and 2
cargo run --bin server 1 &
cargo run --bin server 2 &

# Run test
python3 stress_test.py

# Expected: Load distributed across 2 nodes
```

### Test with 1 Node (Maximum Degradation)

```bash
# Start only Node 3
cargo run --bin server 3 &

# Run test
python3 stress_test.py

# Expected: Node 3 handles all requests
```

## Understanding Hash Distribution

The hash function uses:
```rust
hash = hash(username + filename)
assigned_node = alive_nodes[hash % num_alive_nodes]
```

**Why different filenames matter:**
- `testuser + image_1.png` → hash A → might map to Node 1
- `testuser + image_2.png` → hash B → might map to Node 2
- `testuser + image_3.png` → hash C → might map to Node 3

**Same filename repeatedly:**
- `testuser + test.png` → always same hash → always same node

## Test Parameters

You can modify the test by editing the script:

```python
# In stress_test.py (or stress_test.sh)

NUM_UPLOADS = 30    # Number of different files to upload
USERNAME = "testuser"  # Client username
TEST_IMAGE = "test_image.png"  # Source image
```

More uploads = better statistical distribution analysis

## Output Files

**stress_test_output.log** - Full client output including:
- All upload attempts
- Server responses
- Error messages
- Useful for debugging

## Expected Performance

With 30 uploads:
- **Execution time**: ~10-15 seconds
- **Network**: ~30MB uploaded (30 images × ~1MB each broadcast to 3 servers)
- **Disk**: ~30 encrypted images saved to `images/`

## Cleanup

The test automatically cleans up:
- Temporary test files (deleted)
- Upload commands file (deleted)

**Not cleaned up:**
- Encrypted images in `images/` directory
- `stress_test_output.log` file

To clean manually:
```bash
rm -f images/encrypted_image_*.png
rm -f stress_test_output.log
```

## Summary

Use these stress tests to verify your load balancing is working correctly. A successful test shows even distribution across all alive nodes, proving the fault-tolerant hash-based round-robin algorithm is functioning as designed.
