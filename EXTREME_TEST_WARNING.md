# ⚠️ EXTREME STRESS TEST - READ BEFORE RUNNING ⚠️

## THIS IS AN EXTREME TEST - BE PREPARED!

The `aggressive_stress_test.py` script is configured for **EXTREME load testing**.

## Test Configuration

```
Number of uploads:  1500 images
Image size range:   1600x1600 to 3500x3500 pixels
File size range:    ~5 MB to ~25 MB per image
Total data:         ~15-20 GB
Network traffic:    ~45-60 GB (3x broadcast)
Expected runtime:   30-90 minutes
```

## Resource Requirements

### Minimum System Requirements

**Disk Space:**
- Test images: ~15-20 GB (temporary)
- Encrypted images: ~15-20 GB (permanent)
- **Total needed: 40-50 GB free space**

**Memory:**
- Client: 2-4 GB RAM
- Each server: 1-2 GB RAM
- **Total: 8-10 GB RAM recommended**

**Network:**
- Bandwidth: 100+ Mbps recommended
- Low latency (<10ms between nodes)
- **Will generate ~45-60 GB of traffic**

**Time:**
- Image generation: 5-10 minutes
- Upload phase: 30-80 minutes
- **Total: 35-90 minutes**

## What This Tests

### 1. Massive File Sizes
- **Minimum**: 1600x1600 px = ~5 MB
- **Maximum**: 3500x3500 px = ~25 MB
- **Average**: ~2500x2500 px = ~13 MB

### 2. High Volume
- **1500 unique images**
- **~20 GB of test data**
- **~60 GB network traffic** (each image sent to 3 servers)

### 3. Sustained Performance
- Long-running test (30-90 minutes)
- Finds memory leaks
- Tests network stability
- Validates sustained throughput

### 4. Statistical Significance
- 1500 samples = highly significant
- Load distribution variance clearly visible
- Hash quality measurable to high precision

## Expected Statistics

With 1500 uploads across 3 nodes:

**Perfect balance**: 500 uploads per node (33.33%)
- σ < 1%: Excellent (500 ± 5 per node)
- σ < 2%: Very good (500 ± 10 per node)
- σ < 5%: Good (500 ± 25 per node)

## Before You Run

### 1. Check Disk Space

```bash
df -h .
# You need at least 50 GB free
```

### 2. Check Memory

```bash
free -h
# You need at least 8 GB available
```

### 3. Build in Release Mode

```bash
cargo build --release
```

**IMPORTANT**: Do NOT run servers in debug mode for this test!

### 4. Install Dependencies

```bash
pip3 install --break-system-packages Pillow numpy
```

**Note**: The test requires numpy to generate random pixel data for heavy images.

### 5. Clear Old Test Data

```bash
# Remove old test images
rm -rf images/encrypted_stress_image_*.png
```

## How to Run

### Step 1: Start Servers (Release Mode!)

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

Wait 8 seconds for leader election to complete.

### Step 2: Run Test

**Terminal 4:**
```bash
python3 aggressive_stress_test.py
```

Press ENTER when prompted, then **wait 30-90 minutes**.

## What to Expect

### Generation Phase (5-10 minutes)

```
Creating 1500 test images with varying sizes...
  Size range: 1600x1600 to 3500x3500
  Created 100/1500 images...
  Created 200/1500 images...
  ...
  Created 1500/1500 images...

✓ Created 1500 images in 420.3 seconds
  Distribution:
    - Small  (1600-2000px, ~5-8 MB):    375 images
    - Medium (2000-2500px, ~8-13 MB):   375 images
    - Large  (2500-3000px, ~13-18 MB):  375 images
    - XLarge (3000-3500px, ~18-25 MB):  375 images
  Total size: 18.45 GB (18,894 MB)
  Average size: 12.6 MB per image
  Network traffic (3x broadcast): ~55.4 GB
```

### Upload Phase (30-80 minutes)

Watch server terminals for load distribution:

**Good (balanced):**
```
Node 1: Assigned to me via load balancing (alive nodes: [1, 2, 3])
... (should see ~500 of these on each node)
```

**Bad (imbalanced):**
```
Node 3: Assigned to me via load balancing (alive nodes: [1, 2, 3])
... (all 1500 going to Node 3 only)
```

### Results Phase

```
✓ Test completed in 3,245.7 seconds (54.1 minutes)

Load Distribution (%):
-----------------------------------------------------------
Node 1:  33.2%  ████████████████ (498 uploads, ~6.2 GB)
Node 2:  33.5%  ████████████████ (502 uploads, ~6.3 GB)
Node 3:  33.3%  ████████████████ (500 uploads, ~6.3 GB)

Balance Analysis:
  Expected per node: 33.3%
  Standard deviation: 0.11%
  ✓ Excellent balance (σ < 5%)

Performance Metrics:
  - Throughput: 0.46 uploads/sec
  - Average latency: 2.164 sec/upload
  - Success rate: 100.00%

✓✓✓ SYSTEM PASSED ALL TESTS
```

## Performance Expectations

### Network Speed Impact

| Network | Throughput | Total Time |
|---------|-----------|------------|
| 1 Gbps  | 1.0-2.0 uploads/sec | 12-25 min |
| 100 Mbps | 0.5-1.0 uploads/sec | 25-50 min |
| 10 Mbps | 0.1-0.3 uploads/sec | 80-250 min |

Lower throughput = longer test runtime.

### If Test is Too Slow

You can reduce the scale:

```python
# Edit aggressive_stress_test.py

NUM_UPLOADS = 500        # Instead of 1500
MIN_IMAGE_SIZE = 1600    # Keep large sizes
MAX_IMAGE_SIZE = 2500    # But reduce max
```

This gives: ~500 images × 8 MB avg = ~4 GB data

## Monitoring During Test

### Watch System Resources

**Terminal 5:**
```bash
watch -n 2 'free -h; echo "---"; df -h .; echo "---"; iostat -x 1 1'
```

Look for:
- **Memory**: Should not grow unbounded
- **Disk**: Should have steady free space decreasing
- **I/O**: Should show active read/write

### Watch Network Traffic

**Terminal 6:**
```bash
iftop -i <interface>
# or
nethogs
```

Should see sustained high bandwidth usage.

## Troubleshooting

### "Out of disk space"

**Problem**: Not enough space for images

**Solution**:
```bash
# Check space
df -h .

# Clean old images
rm images/encrypted_stress_image_*.png

# Or reduce NUM_UPLOADS to 500
```

### "Out of memory"

**Problem**: System running out of RAM

**Solution**:
```bash
# Check memory
free -h

# Kill other applications
# Or reduce NUM_UPLOADS to 500
```

### "Test is very slow (<0.2 uploads/sec)"

**Problem**: Network or system bottleneck

**Debug**:
```bash
# Check network latency
ping -c 10 10.40.45.206
ping -c 10 10.40.33.244
ping -c 10 10.40.53.40

# Check if servers are CPU-bound
top

# Check disk I/O
iostat -x 2
```

**Solutions**:
- Reduce image sizes
- Reduce number of uploads
- Use faster network
- Use SSD instead of HDD

### "Connection timeouts"

**Problem**: 100ms health check too aggressive for large images

**Solution**: Edit `src/server.rs`:
```rust
Duration::from_millis(500)  // Increase from 100ms to 500ms
```

## Cleanup After Test

```bash
# Remove test images (done automatically)

# Remove encrypted images to save space
rm -rf images/encrypted_stress_image_*.png

# Remove log file
rm aggressive_stress_test_output.log

# Check space recovered
df -h .
```

## Why This Test Matters

This extreme test validates:

✅ System handles **production-scale files** (5-25 MB each)
✅ System handles **sustained high load** (30-90 minutes)
✅ Load balancing works with **large data volumes**
✅ No memory leaks over **long duration**
✅ Network can handle **60+ GB of traffic**
✅ Hash distribution is **statistically sound** (1500 samples)
✅ Fault tolerance works under **real stress**

## Summary

This is an **EXTREME test**. It will:

- Take **30-90 minutes** to complete
- Generate **~20 GB** of test images
- Transfer **~60 GB** over the network
- Create **~20 GB** of encrypted images
- Use **~8-10 GB** of RAM
- Heavily utilize CPU, network, and disk I/O

**Only run this test if:**
- You have sufficient resources
- You have time to wait (30-90 min)
- You want to validate production-scale performance
- You need statistically significant load distribution data

**For quick testing**, use the basic `stress_test.py` (30 images, 2 minutes).

---

## Ready?

If you have:
- ✅ 50+ GB free disk space
- ✅ 8+ GB available RAM
- ✅ 100+ Mbps network
- ✅ 30-90 minutes to wait
- ✅ Servers running in release mode

Then run:
```bash
python3 aggressive_stress_test.py
```

**Good luck!** 🚀
