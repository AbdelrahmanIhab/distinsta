# Quick Reference Guide

## Milestone Requirements ✅

1. **Image Encryption**: AES-128-CTR with username-derived keys
2. **Load Balancing**: Client multicasts to all servers, distributed election decides handler
3. **Leader Election**: Bully algorithm with automatic failover detection

## Quick Start

```bash
# Build
cargo build --release

# Terminal 1-3: Start servers
cargo run --bin server 1
cargo run --bin server 2
cargo run --bin server 3

# Terminal 4: Start client REPL
cargo run --bin client alice

# In the REPL:
alice> upload test_image.png
alice> quit
```

## Load Balancing Architecture (As Per Specification)

From project description:
> "The client's middleware multicast requests to the cloud servers... Those servers would elect one to carry the incoming workload using a distributed election algorithm"

### Implementation (Efficient!):
1. **Client broadcasts** to all 3 servers (127.0.0.1:8001, 8002, 8003)
2. All servers receive the request
3. **Only leader processes** it (workers reject: "I'm not the leader")
4. Leader encrypts and responds
5. Client filters responses, accepts only successful one
6. **Result**: Only one server does work, distributed coordination!

## Key Features

### 1. REPL Client
- Interactive Read-Eval-Print Loop
- Upload multiple images in one session
- Commands: `upload <path>`, `help`, `quit`

### 2. Multicast with Coordination
- Client automatically sends to all servers
- Only leader processes (workers reject)
- Efficient - no duplicate work!

### 3. Unique Filenames
- Format: `encrypted_<filename>_<timestamp>.<ext>`
- Example: `encrypted_test_image_1730736542.png`
- Multiple uploads don't overwrite each other

### 4. Leader Failover
- Heartbeat every 5 seconds
- Automatic detection of leader failure
- Re-election triggered automatically
- System continues without interruption

## Testing

### Test 1: REPL Basic Upload
```bash
cargo run --bin client alice
alice> upload test_image.png
# Check: images/encrypted_test_image_<timestamp>.png created
alice> quit
```

### Test 2: Multiple Uploads Same File
```bash
cargo run --bin client alice
alice> upload test_image.png
alice> upload test_image.png
# Check: Two different files with different timestamps
alice> quit
```

### Test 3: Leader Failover
```bash
# 1. Start all 3 servers
# 2. Start client REPL: cargo run --bin client bob
# 3. Upload: bob> upload test_image.png
# 4. Kill Node 3 (Ctrl+C)
# 5. Wait ~5-10 seconds (Node 2 becomes leader)
# 6. Upload again: bob> upload test_image.png
# Works with new leader!
bob> quit
```

### Test 4: Different Users
```bash
# Terminal 1:
cargo run --bin client alice
alice> upload test_image.png  # Encrypted with Alice's key
alice> quit

# Terminal 2:
cargo run --bin client bob
bob> upload test_image.png    # Encrypted with Bob's key
bob> quit
# Two files with different encryption keys!
```

## Architecture Diagram

```
                    Client
                      |
        Broadcast to all servers (multicast)
              /       |       \
             v        v        v
        Node 1    Node 2    Node 3 (Leader)
        :8001     :8002     :8003
             \       |       /
              Bully Election
                     |
            Elected node processes
                     |
             Encrypted image
                     |
                 Response
```

## How It Differs from Typical Load Balancing

**Traditional (Centralized):**
- Client → Leader → Leader distributes to workers

**This Implementation (P2P per spec):**
- Client → **All servers** (multicast)
- Servers coordinate via election
- Elected server handles request
- Distributed decision-making

## Common Commands

```bash
# Build
cargo build --release

# Start server node
cargo run --bin server <node_id>   # 1, 2, or 3

# Upload image (broadcasts automatically)
cargo run --bin client <username> <image_file>

# Clean and rebuild
cargo clean && cargo build --release

# Kill all servers
pkill -f "cargo run --bin server"
```

## File Locations

- **Source code**: `src/`
- **Encrypted images**: `images/encrypted_*`
- **Test image**: `test_image.png`
- **Build output**: `target/release/`

## Ports

- Node 1: `127.0.0.1:8001`
- Node 2: `127.0.0.1:8002`
- Node 3: `127.0.0.1:8003`

## Expected Behavior

1. **Startup**: Node 3 becomes leader (highest ID)
2. **Upload**: Client broadcasts to all, leader responds
3. **Save**: Encrypted image saved with timestamp
4. **Failover**: If leader dies, Node 2 takes over in ~5-10 seconds
5. **Continue**: System keeps working with new leader

## Success Criteria

- [ ] All 3 servers start without errors
- [ ] Node 3 elected as initial leader
- [ ] Client broadcasts to all servers
- [ ] Image encrypted and saved with unique filename
- [ ] Leader failure detected within 10 seconds
- [ ] New leader elected automatically
- [ ] System continues working after failover

## Troubleshooting

**"Address already in use"**
```bash
pkill -f "cargo run --bin server"
sleep 2
# Restart servers
```

**"No server responded"**
- Check all 3 servers are running
- Wait 5-8 seconds after starting servers
- Check ports 8001-8003 not blocked

**Images not saving**
- Check `images/` directory exists (created automatically)
- Verify write permissions
- Check disk space

## Summary

This implementation follows the project specification:
- ✅ P2P architecture with multicast
- ✅ Distributed election for load balancing
- ✅ Automatic leader failover
- ✅ Image encryption (AES-128-CTR)
- ✅ Simple, working, educational
