# Distributed System Demo

## Overview

This demo script demonstrates the distributed image storage system with load balancing, fault tolerance, and encryption capabilities.

## Demo Configuration

```
Number of uploads:  50 images
Image size range:   1600x1600 to 3500x3500 pixels
File size range:    ~7 MB to ~37 MB per image
Total data:         ~750 MB to 1 GB
Network traffic:    ~2-3 GB (3x broadcast)
Expected runtime:   2-5 minutes
```

## What This Demo Shows

### 1. Load Balancing
- **Hash-based distribution**: Each image is assigned to a specific node based on `hash(username + filename)`
- **Balanced workload**: All 3 nodes receive approximately equal number of requests
- **Deterministic assignment**: Same request always goes to same node (when available)

### 2. Heavy Image Handling
- **Small**  (1600-2000px): ~7-12 MB each
- **Medium** (2000-2500px): ~12-19 MB each
- **Large**  (2500-3000px): ~19-27 MB each
- **XLarge** (3000-3500px): ~27-37 MB each

### 3. Fault Tolerance
- **Health checking**: Automatic detection of alive nodes
- **Dynamic reassignment**: Requests routed only to healthy nodes
- **Graceful degradation**: System continues with available nodes

### 4. Data Security
- **AES-128-CTR encryption**: All images encrypted before storage
- **Username-derived keys**: Each user has unique encryption keys
- **Secure storage**: Encrypted images stored on assigned nodes

## Prerequisites

### 1. Install Dependencies

```bash
pip3 install --break-system-packages Pillow numpy
```

### 2. Build in Release Mode

```bash
cargo build --release
```

### 3. Start All 3 Servers

**Terminal 1:**
```bash
cargo run --release --bin server 1
```

**Terminal 2:**
```bash
cargo run --release --bin server 2
```

**Terminal 3:**
```bash
cargo run --release --bin server 3
```

**Wait 5-8 seconds** for leader election to complete.

## Running the Demo

**Terminal 4:**
```bash
python3 demo.py
```

Press ENTER when prompted to start.

## Expected Output

### Generation Phase (30-60 seconds)

```
Creating 50 demo images with varying sizes...
  Size range: 1600x1600 to 3500x3500
  Created 10/50 images...
  Created 20/50 images...
  Created 30/50 images...
  Created 40/50 images...
  Created 50/50 images...

✓ Created 50 images in 45.2 seconds
  Distribution:
    - Small  (1600-2000px, ~7-12 MB):  12 images
    - Medium (2000-2500px, ~12-19 MB): 13 images
    - Large  (2500-3000px, ~19-27 MB): 12 images
    - XLarge (3000-3500px, ~27-37 MB): 13 images
  Total size: 892 MB (0.87 GB)
  Average size: 17.8 MB per image
  Network traffic (3x broadcast): ~2.61 GB
```

### Upload Phase (1-3 minutes)

```
Starting Demo Test
Uploading 50 images to demonstrate load balancing...

✓ Demo completed in 123.4 seconds (2.1 minutes)
  Average: 2.468 seconds per upload
  Throughput: 0.41 uploads/sec
```

### Results Analysis

```
Demo Results Analysis

Overall Statistics:
------------------------------------------------------------
Total uploads attempted: 50
Successful uploads:      50 (100.0%)
Failed uploads:          0 (0.0%)
Total time:              123.4 seconds (2.1 minutes)
Throughput:              0.41 uploads/sec

Request Distribution:
------------------------------------------------------------
Node 1: 17 processed, 33 declined
Node 2: 16 processed, 34 declined
Node 3: 17 processed, 33 declined

Total processed: 50

Load Distribution (%):
------------------------------------------------------------
Node 1:  34.0%  █████████████████ (17 uploads)
Node 2:  32.0%  ████████████████ (16 uploads)
Node 3:  34.0%  █████████████████ (17 uploads)

Balance Analysis:
  Expected per node: 33.3%
  Standard deviation: 0.94%
  ✓ Excellent balance (σ < 5%)
```

### Demo Summary

```
Demo Summary

Demo Configuration:
  - Images uploaded: 50
  - Image size range: 1600x1600 to 3500x3500
  - Total time: 123.4 seconds (2.1 minutes)

Performance Metrics:
  - Throughput: 0.41 uploads/sec
  - Average latency: 2.468 sec/upload
  - Success rate: 100.00%

Load Distribution:
  - Node 1:  34.0% (17 uploads)
  - Node 2:  32.0% (16 uploads)
  - Node 3:  34.0% (17 uploads)

Overall Result:
------------------------------------------------------------
  ✓✓✓ DEMO SUCCESSFUL
  The distributed system is working correctly:
    • Load balancing distributes work across all nodes
    • Fault tolerance ensures high availability
    • Hash-based assignment prevents duplicate processing
    • System handles heavy images efficiently
```

## What to Show Your Supervisor

### 1. Load Distribution Statistics
- Point out the excellent balance (σ < 5%)
- Show that all 3 nodes are processing requests
- Explain the hash-based distribution algorithm

### 2. Heavy Image Handling
- Highlight the variety of image sizes (7-37 MB)
- Show that the system handles ~1 GB of data efficiently
- Mention network traffic (~3 GB broadcast to all nodes)

### 3. Encrypted Storage
```bash
ls -lh images/encrypted_demo_image_*.png
```
- Show the encrypted images in the `images/` directory
- Explain AES-128-CTR encryption
- Username-derived keys for security

### 4. Server Logs
Watch the server terminals during the demo:
- Show "Assigned to me via load balancing" messages
- Point out health checking ("alive nodes: [1, 2, 3]")
- Demonstrate rejections when not assigned

### 5. Fault Tolerance (Optional)
If time permits, demonstrate fault tolerance:

1. Stop one server (Ctrl+C in its terminal)
2. Run demo again
3. Show that system continues with 2 nodes
4. Restart the stopped server
5. Show automatic recovery

## Performance Expectations

| Network Speed | Throughput | Demo Time |
|---------------|-----------|-----------|
| 1 Gbps        | 1-2 uploads/sec | 30-50 sec |
| 100 Mbps      | 0.3-0.8 uploads/sec | 1-3 min |
| 10 Mbps       | 0.1-0.3 uploads/sec | 3-8 min |

## Troubleshooting

### "No servers running"

Check that all 3 servers are started:
```bash
cargo run --release --bin server 1
cargo run --release --bin server 2
cargo run --release --bin server 3
```

### "Connection timeout"

The 100ms health check might be too aggressive. Wait a few seconds between server startup and running the demo.

### "Out of disk space"

Demo needs ~3-4 GB free space:
```bash
df -h .
```

Clean old demo images:
```bash
rm -rf images/encrypted_demo_image_*.png
```

## Cleanup After Demo

```bash
# Remove encrypted demo images
rm -rf images/encrypted_demo_image_*.png

# Remove log file
rm demo_output.log
```

## Key Points for Presentation

1. **Load Balancing**: Hash-based distribution ensures even workload across all nodes
2. **Fault Tolerance**: System automatically detects and excludes dead nodes
3. **Scalability**: Handles heavy images (7-37 MB) efficiently
4. **Security**: AES-128-CTR encryption with username-derived keys
5. **Reliability**: 100% success rate with proper node health
6. **Performance**: Sustained throughput of 0.3-2.0 uploads/sec depending on network

---

**Good luck with your presentation!** 🚀
