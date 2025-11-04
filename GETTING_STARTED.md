# Getting Started - Quick Reference

## Prerequisites

- Rust (1.70 or later): Install from https://rustup.rs/
- 4 terminal windows
- About 5 minutes

## Quick Start (5 Steps)

### Step 1: Build (1 minute)

```bash
cd /home/abdelrahmanelazab/distinsta
cargo build --release
```

### Step 2: Start Server Nodes (30 seconds)

Open **Terminal 1**:
```bash
cargo run --bin server 1
```

Open **Terminal 2**:
```bash
cargo run --bin server 2
```

Open **Terminal 3**:
```bash
cargo run --bin server 3
```

Wait for this output:
```
Node 3: I am the LEADER, initializing load balancer
```

### Step 3: Start Client (10 seconds)

Open **Terminal 4**:
```bash
cargo run --bin client alice 127.0.0.1:8003
```

### Step 4: Upload Image (30 seconds)

In the client menu:
```
Choose option: 1
Enter image file path: test_image.png
```

You should see:
```
Success: Image test_image.png uploaded and encrypted successfully
```

### Step 5: Download Image (30 seconds)

```
Choose option: 2
Enter image filename: test_image.png
Enter output file path: downloaded.png
```

Verify the downloaded image:
```bash
ls -lh downloaded.png
```

## What Just Happened?

1. **Leader Election**: Node 3 won the election (highest ID)
2. **Encryption**: Your image was encrypted with AES using "alice" as the key
3. **Steganography**: Encrypted data was hidden in the image pixels
4. **Storage**: Stego-image stored in Node 3's memory
5. **Decryption**: Image retrieved and decrypted back to original

## Try This Next

### Test Different Users

```bash
# Terminal 5 - User Bob
cargo run --bin client bob 127.0.0.1:8001

# Upload his own image
Choose option: 1
Enter image file path: test_image.png

# Try to download Alice's image (will fail to decrypt properly!)
Choose option: 2
Enter image filename: test_image.png
Enter output file path: bob_wrong.png
```

This demonstrates that each user's images are encrypted with their own key!

### List All Images

```
Choose option: 3
```

This shows all images you've uploaded.

## Troubleshooting

**"Address already in use"**
```bash
pkill -f "cargo run --bin server"
# Wait 5 seconds, then restart servers
```

**"Connection refused"**
- Make sure all 3 servers are running
- Check they show "listening on 127.0.0.1:800X"

**"Image not found"**
- Make sure you uploaded as the same username
- Images are per-user!

## Understanding the System

### Ports
- Node 1: `127.0.0.1:8001` (Worker)
- Node 2: `127.0.0.1:8002` (Worker)
- Node 3: `127.0.0.1:8003` (Leader)

### Files
- `test_image.png`: Sample 800x600 blue image
- `downloaded.png`: Your retrieved image
- `src/`: Source code modules

### Key Concepts

**Bully Algorithm**
- Node 3 has highest ID → becomes leader
- If leader fails, new election would occur
- Leader coordinates all operations

**Image Encryption**
- AES-128-CTR: Fast stream cipher
- Key = SHA256(username)[:16]
- Each user gets unique encryption key

**Steganography**
- LSB (Least Significant Bit) method
- Hides data in image pixels
- Imperceptible visual changes
- Original image used as "cover"

**Load Balancing**
- Round-robin distribution
- Leader tracks worker availability
- Fair request distribution

## Next Steps

1. Read [README.md](README.md) for architecture details
2. Read [USAGE.md](USAGE.md) for more examples
3. Read [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) for technical details
4. Explore the source code in `src/`

## Source Code Overview

```
src/
├── server.rs         - Main server logic, handles connections
├── client.rs         - Interactive client interface
├── bully.rs          - Bully election algorithm
├── loadbalancer.rs   - Request distribution
├── encryption.rs     - AES + steganography
└── protocol.rs       - Message formats
```

## Common Commands

```bash
# Build in debug mode
cargo build

# Build optimized
cargo build --release

# Run specific binary
cargo run --bin server 1
cargo run --bin client alice 127.0.0.1:8003

# Clean build artifacts
cargo clean

# Check for errors without building
cargo check

# View documentation
cargo doc --open
```

## Expected Output

### Server (Node 3 - Leader)
```
Starting Server Node 3 on 127.0.0.1:8003
Node 3 listening on 127.0.0.1:8003
Node 3: Starting initial election
Node 3: Starting election
Node 3: I am the new leader!
Node 3: New leader is Node 3
Node 3: I am the LEADER, initializing load balancer
```

### Client
```
=== Distributed Image Storage Client ===
User: alice
Connected to: 127.0.0.1:8003

1. Upload image
2. Download image
3. List images
4. Exit

Choose an option: 1
Enter image file path: test_image.png
Uploading image: test_image.png
Success: Image test_image.png uploaded and encrypted successfully
```

## Performance Tips

- First build takes ~1-2 minutes (downloads dependencies)
- Subsequent builds are much faster (~10 seconds)
- Release builds are optimized but take longer to compile
- Debug builds are faster to compile but slower to run

## Support

If something doesn't work:

1. Check Rust version: `rustc --version` (should be 1.70+)
2. Update Rust: `rustup update`
3. Clean and rebuild: `cargo clean && cargo build`
4. Check all 3 servers are running
5. Verify ports 8001-8003 are not in use
6. Review error messages carefully

## Success Checklist

- [ ] Rust installed and working
- [ ] Project builds without errors
- [ ] 3 servers start successfully
- [ ] Node 3 becomes leader
- [ ] Client connects to server
- [ ] Image uploads successfully
- [ ] Image downloads successfully
- [ ] Downloaded image matches original

If all checkboxes are checked, your distributed system is working correctly!

## What's Impressive About This

This simple system demonstrates:
- Distributed consensus (leader election)
- Load balancing across nodes
- Peer-to-peer communication
- Cryptographic security
- Image processing
- Async/concurrent networking
- All in ~500 lines of Rust code!

Enjoy exploring your distributed system!
