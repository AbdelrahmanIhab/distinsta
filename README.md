# Distributed Image Storage System

A simple distributed system in Rust featuring image encryption, load balancing, and leader election with automatic failover.

## Milestone Requirements

✅ **Image Encryption**: AES-128-CTR encryption with username-derived keys
✅ **Load Balancing**: Round-robin request distribution by the leader node
✅ **Leader Election**: Bully algorithm with automatic re-election on leader failure

## Quick Start

### 1. Build the Project

```bash
cargo build --release
```

### 2. Start the Server Nodes

Open **3 separate terminal windows**:

```bash
# Terminal 1 - Node 1
cargo run --bin server 1

# Terminal 2 - Node 2
cargo run --bin server 2

# Terminal 3 - Node 3 (will become leader)
cargo run --bin server 3
```

Wait ~5-8 seconds for:
- Leader election to complete
- Heartbeat monitoring to start

You should see:
```
Node 3: I am the new leader!
Node 3: I am the LEADER, initializing load balancer
```

### 3. Start the Client (REPL)

Open a **4th terminal**:

```bash
cargo run --bin client alice
```

You'll see an interactive prompt:

```
=== Distributed Image Storage Client (REPL) ===
User: alice
Multicast mode: Broadcasting to all servers
Type 'help' for commands, 'quit' to exit
================================================

alice> upload test_image.png
```

The upload process:
1. Client **broadcasts** to all 3 servers
2. **Only the leader processes** the request (others reject it)
3. Leader encrypts and returns the image
4. Client saves to `images/encrypted_test_image_<timestamp>.png`

## How It Works

### Image Encryption (AES-128-CTR)

1. Client **broadcasts** image to all server nodes simultaneously
2. Servers receive the request and coordinate (via leader election)
3. Elected server processes the request
4. Server generates encryption key: `key = SHA256(username)[:16]`
5. Server encrypts image data with AES-128-CTR
6. Server returns **encrypted image** to client
7. Client saves encrypted image to `images/` with unique timestamp

**No steganography** - just direct AES encryption!

### Load Balancing (P2P + Multicast)

**Efficient distributed decision-making:**
1. **Client multicasts** requests to all servers (P2P architecture)
2. All servers receive the request
3. **Only the leader processes** it (workers reject with error)
4. Leader encrypts and responds
5. Workers return error: "Node X is not the leader"
6. Client accepts only the successful response from leader

This ensures **only one server does the work** while maintaining distributed architecture!

### Leader Election (Bully Algorithm)

**Initial Election:**
- All nodes start and perform election
- Node with highest ID wins (Node 3)
- Leader announces to all peers

**Failover (NEW!):**
- Workers send heartbeats to leader every 5 seconds
- If leader doesn't respond → **new election triggered**
- Next highest node becomes leader automatically

## Testing Leader Failover

1. Start all 3 nodes as above
2. Wait for Node 3 to become leader
3. **Kill Node 3** (`Ctrl+C` in its terminal)
4. Watch Node 2 detect the failure and start election:
   ```
   Node 2: Leader 3 is DOWN! Starting new election...
   Node 2: I am the new leader!
   ```
5. Node 2 is now the new leader!

## Architecture

```
Client (alice)
    |
    v
┌──────────────────────────────────────┐
│   Node 3 (Leader) - Port 8003       │
│   - Bully Election                   │
│   - Heartbeat Monitoring             │
│   - Load Balancer                    │
│   - AES Encryption                   │
└────────┬─────────────────────────────┘
         │
    ┌────┴────┐
    v         v
┌─────────┐ ┌─────────┐
│ Node 1  │ │ Node 2  │
│ Worker  │ │ Worker  │
│ 8001    │ │ 8002    │
└─────────┘ └─────────┘
```

## Client Usage (REPL)

```bash
cargo run --bin client <username>
```

The client starts an interactive REPL (Read-Eval-Print Loop).

**Example Session:**

```bash
$ cargo run --bin client alice

=== Distributed Image Storage Client (REPL) ===
User: alice
Multicast mode: Broadcasting to all servers
Type 'help' for commands, 'quit' to exit
================================================

alice> help
Available commands:
  upload <image_path>  - Upload and encrypt an image
  help                 - Show this help message
  quit                 - Exit the client

alice> upload test_image.png
Broadcasting request to 3 servers...
  Sending to server 1 at 127.0.0.1:8001
  Sending to server 2 at 127.0.0.1:8002
  Sending to server 3 at 127.0.0.1:8003
  - Server 1 declined: Node 1 is not the leader
  - Server 2 declined: Node 2 is not the leader
  ✓ Leader (server 3) processed request
✓ Success!
Encrypted image saved to: images/encrypted_test_image_1730738456.png

alice> upload another_image.jpg
...

alice> quit
Goodbye!
```

Each user gets a unique encryption key!
Each upload gets a unique timestamped filename.

## Project Structure

```
src/
├── server.rs         # Server node with leader/worker modes
├── client.rs         # Simple upload-only client
├── bully.rs          # Bully algorithm + heartbeat monitoring
├── loadbalancer.rs   # Round-robin load distribution
├── encryption.rs     # AES-128-CTR encryption
└── protocol.rs       # Message protocol definitions

images/               # Encrypted images saved here
test_image.png        # Sample test image
```

## Key Features

### 1. Image Encryption
- **AES-128-CTR** stream cipher
- **Username-based keys**: `SHA256(username)[:16]`
- **Direct encryption**: No steganography complexity
- **Fast**: Suitable for large images

### 2. Load Balancing
- Leader maintains worker pool
- Round-robin request distribution
- Health tracking

### 3. Leader Election with Failover
- **Bully algorithm**: Highest ID wins
- **Heartbeat monitoring**: Checks leader every 5 seconds
- **Automatic re-election**: If leader fails, workers detect and elect new leader
- **No single point of failure**: System recovers automatically

## Example Session

```bash
# Terminal 1-3: Start servers
$ cargo run --bin server 1 &
$ cargo run --bin server 2 &
$ cargo run --bin server 3 &

# Wait 8 seconds...
# Node 3: I am the LEADER

# Terminal 4: Start client REPL
$ cargo run --bin client alice

alice> upload test_image.png
Broadcasting request to 3 servers...
  Sending to server 1 at 127.0.0.1:8001
  Sending to server 2 at 127.0.0.1:8002
  Sending to server 3 at 127.0.0.1:8003
  - Server 1 declined: Node 1 is not the leader
  - Server 2 declined: Node 2 is not the leader
  ✓ Leader (server 3) processed request
✓ Success!
Encrypted image saved to: images/encrypted_test_image_1730738456.png

# Now kill Node 3 (Ctrl+C in its terminal)
# Node 2: Leader 3 is DOWN! Starting new election...
# Node 2: I am the new leader!

# Upload still works with new leader!
alice> upload test_image.png
Broadcasting request to 3 servers...
  - Server 1 declined: Node 1 is not the leader
  ✓ Leader (server 2) processed request  # Node 2 is now leader!
✓ Success!
Encrypted image saved to: images/encrypted_test_image_1730738512.png

alice> quit
Goodbye!
```

## Technical Details

### Dependencies
- `tokio`: Async runtime
- `serde`/`serde_json`: Message serialization
- `aes`/`ctr`: AES encryption
- `sha2`: Key derivation

### Network Protocol
- **TCP connections** on ports 8001-8003
- **JSON messages** for simplicity
- **Async I/O** with Tokio

### Security Notes

⚠️ **Educational Implementation**

This is for learning purposes. Production systems need:
- Proper key management (not username-derived)
- TLS for network encryption
- Authentication/authorization
- Persistent storage
- Better error handling

## Troubleshooting

**"Address already in use"**
```bash
pkill -f "cargo run --bin server"
# Wait 2 seconds, then restart
```

**"Connection refused"**
- Ensure all 3 servers are running
- Wait for election to complete (~5-8 seconds)

**No encrypted image saved**
- Check `images/` directory was created
- Verify file permissions

## Testing Checklist

- [ ] All 3 servers start
- [ ] Node 3 becomes initial leader
- [ ] Client uploads image successfully
- [ ] Encrypted image saved to `images/`
- [ ] Kill leader (Node 3)
- [ ] Node 2 becomes new leader
- [ ] Client still works with new leader

## Key Implementation Details

✅ **Multicast/Broadcast**: Client sends to all servers (P2P architecture as specified)
✅ **Leader Failover**: Automatic re-election with heartbeat monitoring
✅ **Unique Filenames**: Timestamp-based naming prevents overwrites
✅ **Direct Encryption**: AES-128-CTR (no steganography complexity)
✅ **Distributed Election**: Bully algorithm for coordinator selection

## Summary

This distributed system demonstrates:
1. **Image Encryption** - AES-128-CTR with per-user keys
2. **Load Balancing** - Round-robin by leader
3. **Leader Election** - Bully algorithm with **automatic failover**

Simple, working, and educational!
