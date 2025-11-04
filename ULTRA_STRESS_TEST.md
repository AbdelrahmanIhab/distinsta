# Ultra-Aggressive Stress Test

## Overview

This test pushes the distributed system to its limits with **2000 large images**.

## Configuration

```python
NUM_UPLOADS = 2000          # Total images to upload
MIN_IMAGE_SIZE = 100        # 100x100 pixels
MAX_IMAGE_SIZE = 2000       # 2000x2000 pixels (up to 4MB per image)
```

## What This Tests

### 1. Volume Testing
- **2000 unique images**
- Each with different:
  - Size (100px to 2000px)
  - Color
  - Content (unique number)
  - Hash value

### 2. Size Distribution
- **Small** (100-500px): ~500 images (~0.1-1 MB each)
- **Medium** (500-1000px): ~500 images (~1-4 MB each)
- **Large** (1000-1500px): ~500 images (~4-9 MB each)
- **XLarge** (1500-2000px): ~500 images (~9-16 MB each)

**Expected total data: ~10-20 GB** (broadcasted to 3 servers = 30-60 GB network traffic)

### 3. Performance Under Load
- Sustained high throughput
- Network bandwidth saturation
- Memory usage under load
- Concurrent connection handling
- Hash function distribution quality

### 4. Statistical Validation
- With 2000 data points, hash distribution quality is highly measurable
- Standard deviation should be < 2% for excellent balance
- Any imbalance will be clearly visible

## Expected Execution Time

**Generation phase**: 2-5 minutes
- Creating 2000 unique PNG images

**Upload phase**: 10-30 minutes (depends on network)
- 2000 uploads × (broadcast to 3 servers)
- Total: 6000 TCP connections
- 30-60 GB network traffic

**Total test time**: ~15-35 minutes

## Resource Requirements

### Disk Space
- **Test images**: ~5-10 GB (temporary)
- **Encrypted images**: ~5-10 GB (in `images/` directory)
- **Total**: 10-20 GB free space needed

### Memory
- **Client**: ~1-2 GB (buffering large images)
- **Each server**: ~500 MB - 1 GB
- **Total system**: ~3-5 GB RAM recommended

### Network
- **Bandwidth**: Each image broadcasted 3x
- **Connections**: 6000 TCP connections total
- **Latency**: Critical for throughput

## How to Run

### Prerequisites

```bash
# Install dependencies
pip3 install --break-system-packages Pillow numpy

# Ensure sufficient disk space
df -h .

# Build in release mode for performance
cargo build --release
```

### Start All Servers

```bash
# Terminal 1
cargo run --release --bin server 1

# Terminal 2
cargo run --release --bin server 2

# Terminal 3
cargo run --release --bin server 3

# Wait 5-8 seconds for leader election
```

### Run the Test

```bash
# Terminal 4
python3 aggressive_stress_test.py

# Press ENTER when prompted
```

## What to Monitor

### During Test

**Server terminals** - Watch for:
- "Assigned to me via load balancing (alive nodes: [1, 2, 3])"
- Each server should show ~666-667 "Assigned to me" messages
- Each server should show ~1333-1334 "Request assigned to Node X" rejections

**System resources**:
```bash
# Monitor in another terminal
watch -n 1 'free -h; echo "---"; df -h .'
```

### Expected Good Output

```
Node 1: Assigned to me via load balancing (alive nodes: [1, 2, 3])
Node 1: Processing image upload...
Node 1: Image encrypted (1234567 bytes -> 1234567 bytes)
... (should see ~667 of these)

Node 2: Request assigned to Node 1 (round-robin), rejecting
Node 2: Assigned to me via load balancing (alive nodes: [1, 2, 3])
Node 2: Processing image upload...
... (should see ~667 of these)

Node 3: Similar pattern (~666 assignments)
```

### Bad Output (Load Balancing Broken)

```
Node 3: Assigned to me via load balancing (alive nodes: [1, 2, 3])
Node 3: Processing image upload...
... (all 2000 going to Node 3 only)
```

## Expected Results

### Perfect Load Balance

```
Load Distribution (%):
-----------------------------------------------------------
Node 1:  33.3%  ████████████████ (666 uploads)
Node 2:  33.4%  ████████████████ (667 uploads)
Node 3:  33.3%  ████████████████ (667 uploads)

Balance Analysis:
  Expected per node: 33.3%
  Standard deviation: 0.05%
  ✓ Excellent balance (σ < 5%)

Performance Metrics:
  - Throughput: 2.5 uploads/sec
  - Average latency: 0.400 sec/upload
  - Success rate: 100.00%

✓✓✓ SYSTEM PASSED ALL TESTS
```

### Imbalanced (But Working)

```
Load Distribution (%):
-----------------------------------------------------------
Node 1:  31.2%  ███████████████ (624 uploads)
Node 2:  35.6%  █████████████████ (712 uploads)
Node 3:  33.2%  ████████████████ (664 uploads)

Balance Analysis:
  Expected per node: 33.3%
  Standard deviation: 1.85%
  ✓ Good balance (σ < 5%)

⚠ Distribution is acceptable but not perfect
```

### Broken Load Balancing

```
Load Distribution (%):
-----------------------------------------------------------
Node 1:   0.0%   (0 uploads)
Node 2:   0.0%   (0 uploads)
Node 3: 100.0%  ██████████████████████████ (2000 uploads)

Balance Analysis:
  Expected per node: 33.3%
  Standard deviation: 47.14%
  ✗ Poor balance (σ >= 15%)

✗ FAIL: Only 1 node is processing requests
✗ Load balancing is NOT working!
```

## Interpreting Statistics

### Standard Deviation (σ)

The key metric for balance quality:

- **σ < 1%**: Perfect (near-ideal hash distribution)
- **σ < 5%**: Excellent (production-ready)
- **σ < 10%**: Good (acceptable variance)
- **σ < 15%**: Acceptable (some imbalance)
- **σ >= 15%**: Poor (load balancing issues)

With 2000 samples, even small imbalances are statistically significant.

### Throughput

Expected values:
- **Local network**: 5-10 uploads/sec
- **Distributed network**: 1-5 uploads/sec
- **Slow network**: 0.5-1 uploads/sec

Lower throughput indicates:
- Network bottleneck
- Server CPU overload
- Disk I/O saturation

## Troubleshooting

### "Test timed out after 20 minutes"

**Cause**: System too slow or hung

**Solutions**:
- Check if all servers are responsive
- Monitor network latency
- Check disk space (may be full)
- Reduce NUM_UPLOADS to 500 or 1000

### "Out of memory"

**Cause**: System running out of RAM

**Solutions**:
```bash
# Check memory
free -h

# Kill other processes
# Or reduce NUM_UPLOADS
```

### "Out of disk space"

**Cause**: Not enough space for images

**Solutions**:
```bash
# Check space
df -h .

# Clean old encrypted images
rm images/encrypted_stress_image_*.png

# Or reduce NUM_UPLOADS
```

### Very Low Throughput (<0.5 uploads/sec)

**Possible causes**:
- Network latency too high
- Servers CPU-bound
- Disk I/O bottleneck
- Health check timeout (100ms) too aggressive

**Debug**:
```bash
# Check network latency between nodes
ping 10.40.45.206
ping 10.40.33.244
ping 10.40.53.40

# Check CPU usage
top

# Check disk I/O
iostat -x 1
```

## Cleanup

After the test:

```bash
# Remove temporary test images (done automatically)
# Remove encrypted images to save space
rm -rf images/encrypted_stress_image_*.png

# Remove log file
rm aggressive_stress_test_output.log
```

## Performance Optimization Tips

### If test is too slow

1. **Build in release mode**:
   ```bash
   cargo build --release
   cargo run --release --bin server 1
   ```

2. **Reduce image count**:
   ```python
   NUM_UPLOADS = 500  # Instead of 2000
   ```

3. **Reduce max image size**:
   ```python
   MAX_IMAGE_SIZE = 1000  # Instead of 2000
   ```

4. **Increase health check timeout** (in server.rs):
   ```rust
   Duration::from_millis(200)  // Instead of 100
   ```

## What Success Looks Like

A successful test proves:

✅ Load balancing **actually works** (all 3 nodes used)
✅ Hash function **distributes evenly** (σ < 5%)
✅ System handles **high volume** (2000 uploads)
✅ System handles **large images** (up to 2000x2000)
✅ System is **reliable** (>95% success rate)
✅ System is **performant** (reasonable throughput)
✅ Health checking works (detects alive nodes)
✅ Fault tolerance works (degrades gracefully)

## Comparison: Basic vs Aggressive Test

| Metric | Basic (30 images) | Aggressive (2000 images) |
|--------|-------------------|--------------------------|
| Sample size | 30 | 2000 |
| Statistical power | Low | High |
| Image sizes | Same | Varied (100-2000px) |
| Total data | ~10 MB | ~10-20 GB |
| Run time | ~30 sec | ~15-35 min |
| Disk space | ~30 MB | ~20 GB |
| Hash quality | Not measurable | Very measurable |
| Stress level | Light | Extreme |

## Final Notes

This test is **extreme** and will:
- Stress all components of the system
- Generate significant network traffic
- Use significant disk space
- Take considerable time to complete

Use it to validate that your system can handle **production-scale load** with proper distribution across all nodes.

If this test passes with good statistics, your load balancing implementation is **production-ready**!
