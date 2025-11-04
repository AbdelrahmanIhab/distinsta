# Usage Guide

## Quick Start

### 1. Build the Project

```bash
cargo build --release
```

### 2. Run the Servers

Open **3 separate terminal windows** and run each command:

**Terminal 1 - Node 1:**
```bash
cargo run --bin server 1
```

**Terminal 2 - Node 2:**
```bash
cargo run --bin server 2
```

**Terminal 3 - Node 3 (will become leader):**
```bash
cargo run --bin server 3
```

Wait about 5 seconds for the leader election to complete. You should see messages like:
```
Node 3: I am the LEADER, initializing load balancer
```

### 3. Run the Client

Open a **4th terminal window**:

```bash
cargo run --bin client alice 127.0.0.1:8003
```

Replace `alice` with any username you want. You can connect to any node (8001, 8002, or 8003).

### 4. Using the Client

You'll see an interactive menu:

```
=== Distributed Image Storage Client ===
User: alice
Connected to: 127.0.0.1:8003

1. Upload image
2. Download image
3. List images
4. Exit

Choose an option:
```

#### Upload an Image

1. Choose option `1`
2. Enter the path to your image: `test_image.png`
3. The image will be encrypted with your username as the key

#### List Your Images

1. Choose option `3`
2. You'll see all images you've uploaded

#### Download an Image

1. Choose option `2`
2. Enter the filename: `test_image.png`
3. Enter where to save it: `downloaded_image.png`
4. The image will be decrypted and saved

## Testing Different Users

You can run multiple clients with different usernames:

```bash
# Terminal 4 - User Alice
cargo run --bin client alice 127.0.0.1:8003

# Terminal 5 - User Bob
cargo run --bin client bob 127.0.0.1:8001

# Terminal 6 - User Charlie
cargo run --bin client charlie 127.0.0.1:8002
```

Each user can only decrypt their own images!

## How It Works

### Leader Election (Bully Algorithm)

1. When servers start, they perform an election
2. Node with highest ID becomes the leader (Node 3)
3. Leader coordinates requests and manages load balancing

### Image Encryption

1. Image data is encrypted using AES-128-CTR
2. Encryption key is derived from username using SHA-256
3. Encrypted data is hidden in the image using LSB steganography
4. Only the user with the correct username can decrypt

### Load Balancing

- Leader maintains a list of available workers
- Uses round-robin to distribute requests
- Tracks server load and availability

## Example Session

```bash
# Start all 3 servers first (in separate terminals)
cargo run --bin server 1
cargo run --bin server 2
cargo run --bin server 3

# Wait 5 seconds, then start client
cargo run --bin client alice 127.0.0.1:8003

# In client menu:
Choose option: 1
Enter image file path: test_image.png
# Success: Image test_image.png uploaded and encrypted successfully

Choose option: 3
# Images:
#   - test_image.png

Choose option: 2
Enter image filename: test_image.png
Enter output file path: retrieved.png
# Image saved to: retrieved.png

Choose option: 4
# Goodbye!
```

## Verifying the Encryption

Try downloading an image with a different username:

```bash
# Upload as Alice
cargo run --bin client alice 127.0.0.1:8003
# Upload test_image.png

# Try to download as Bob
cargo run --bin client bob 127.0.0.1:8003
# Try to download test_image.png
# You'll get garbled data because Bob's key is different!
```

## Troubleshooting

### "Address already in use"
- One of the servers is already running
- Kill existing processes: `pkill -f "cargo run --bin server"`

### "Connection refused"
- Servers aren't running yet
- Make sure all 3 servers are started before running client

### "Image not found"
- Make sure you're using the correct username
- Images are stored per-user

### Build errors
- Make sure you have Rust installed: `rustc --version`
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean && cargo build`

## Architecture Overview

```
┌─────────────┐
│   Client    │
│  (alice)    │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│          Server Node 3              │
│        (LEADER - Port 8003)         │
│  ┌─────────────────────────────┐   │
│  │    Bully Election           │   │
│  │    Load Balancer            │   │
│  │    Image Processing         │   │
│  └─────────────────────────────┘   │
└──────────┬──────────────────────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
┌─────────┐  ┌─────────┐
│ Node 1  │  │ Node 2  │
│ Worker  │  │ Worker  │
│ 8001    │  │ 8002    │
└─────────┘  └─────────┘
```

## Features Demonstrated

1. **Image Encryption**: AES encryption + LSB steganography
2. **Load Balancing**: Round-robin request distribution
3. **Leader Election**: Bully algorithm with automatic failover
4. **P2P Architecture**: Nodes communicate directly
5. **User Isolation**: Each user's images encrypted with unique key
