# Fault-Tolerant Load Balancing

## Overview

This system implements **deterministic hash-based round-robin load balancing** with automatic node failure detection and recovery.

## How It Works

### 1. Request Assignment

Each request is assigned to a server using a **deterministic hash**:

```rust
hash = hash(username + filename)
assigned_server = alive_nodes[hash % num_alive_nodes]
```

**Key Properties:**
- Same request (same user + filename) → Same server (if alive)
- Different requests → Distributed across servers
- Deterministic: No random assignment
- Consistent: Multiple clients see same assignment

### 2. Node Health Detection

Before processing, each server:

1. **Checks which peers are alive** (100ms connection timeout)
2. **Builds list of available nodes** (including itself)
3. **Calculates assignment** based on alive nodes only
4. **Processes or rejects** based on assignment

```
Every request triggers:
┌─────────────────────────────────┐
│ 1. Quick health check (100ms)  │
│ 2. Build alive_nodes list      │
│ 3. Calculate: hash % alive     │
│ 4. Process if assigned to me   │
└─────────────────────────────────┘
```

### 3. Fault Tolerance

**Scenario: Node 3 crashes**

```
Before crash:
  alive_nodes = [1, 2, 3]
  Request A: hash % 3 = 2 → Node 3 processes
  Request B: hash % 3 = 0 → Node 1 processes
  Request C: hash % 3 = 1 → Node 2 processes

After Node 3 crashes:
  alive_nodes = [1, 2]  ← Node 3 excluded
  Request A: hash % 2 = 0 → Node 1 processes  ✓ Reassigned!
  Request B: hash % 2 = 0 → Node 1 processes
  Request C: hash % 2 = 1 → Node 2 processes
```

**Automatic Recovery:**
- Dead nodes excluded from alive list
- Requests redistributed to remaining nodes
- No manual intervention needed
- Works even with 1 node remaining

## Load Distribution

### With All 3 Nodes Alive

Requests are distributed approximately evenly:

```
100 requests with 3 nodes:
  Node 1: ~33 requests
  Node 2: ~33 requests
  Node 3: ~34 requests
```

### With 2 Nodes Alive

Load is redistributed:

```
100 requests with 2 nodes (Node 3 down):
  Node 1: ~50 requests
  Node 2: ~50 requests
  Node 3: 0 requests (down)
```

### With 1 Node Alive

Single node handles everything:

```
100 requests with 1 node (Node 2, 3 down):
  Node 1: 100 requests
  Node 2: 0 requests (down)
  Node 3: 0 requests (down)
```

## Architecture

### Request Flow

```
Client sends request
        ↓
┌───────────────────────┐
│ Broadcast to all 3    │
│ servers simultaneously│
└───────┬───────────────┘
        │
   ┌────┴────┐
   ↓         ↓         ↓
┌─────┐  ┌─────┐  ┌─────┐
│ N1  │  │ N2  │  │ N3  │ ← Each checks alive nodes
└──┬──┘  └──┬──┘  └──┬──┘
   │        │        │
   ├─ Check peers alive (100ms timeout)
   ├─ Calculate: hash % alive_count
   └─ Process if assigned to me
        │
        ↓
Only 1 node processes
Others reject
```

### Health Check Mechanism

Each server performs **quick connection test** (100ms timeout):

```rust
for each peer {
    try connect with 100ms timeout
    if success: mark as alive
    if timeout/fail: mark as down
}
```

**Why 100ms timeout?**
- Fast detection (doesn't slow down requests)
- Sufficient for local network
- Quick enough to detect crashes
- Prevents long hangs

## Advantages

### 1. True Load Distribution
✓ Work distributed across **all alive nodes**
✓ No single node bottleneck
✓ Scales with number of nodes

### 2. Fault Tolerance
✓ Automatic dead node detection
✓ Instant redistribution to alive nodes
✓ No requests lost
✓ Works with any number of alive nodes (1-3)

### 3. Deterministic Assignment
✓ Same request → same server (when possible)
✓ Predictable behavior
✓ Easy to debug

### 4. Simple & Efficient
✓ No central coordinator needed
✓ Fast health checks (100ms)
✓ Low overhead per request
✓ Stateless (no shared counter to sync)

## Comparison with Previous Approaches

### Leader-Only (Old)
- ✗ Single node bottleneck
- ✓ Simple
- ✓ No race conditions
- ✗ Wasted resources (2 idle nodes)

### Timestamp-Based (Old)
- ✓ Distributed work
- ✗ Could assign to dead nodes
- ✗ Race conditions possible
- ✗ No health awareness

### Hash-Based with Health Check (Current)
- ✓ Distributed work
- ✓ Dead node detection
- ✓ No race conditions (deterministic hash)
- ✓ Automatic failover
- ✓ Fault tolerant

## Example Scenarios

### Scenario 1: Normal Operation (3 Nodes)

```bash
alice> upload image1.png
# hash(alice + image1.png) % 3 = 1
# → Node 2 processes

alice> upload image2.png
# hash(alice + image2.png) % 3 = 0
# → Node 1 processes

alice> upload image3.png
# hash(alice + image3.png) % 3 = 2
# → Node 3 processes
```

Result: **Work distributed evenly**

### Scenario 2: Node Failure (2 Nodes)

```bash
# Node 3 crashes

alice> upload image1.png
# alive_nodes = [1, 2]
# hash(alice + image1.png) % 2 = 1
# → Node 2 processes ✓

alice> upload image2.png
# alive_nodes = [1, 2]
# hash(alice + image2.png) % 2 = 0
# → Node 1 processes ✓
```

Result: **Requests redistributed automatically, no failures**

### Scenario 3: Single Node Remaining

```bash
# Only Node 1 alive

alice> upload image1.png
# alive_nodes = [1]
# hash(alice + image1.png) % 1 = 0
# → Node 1 processes ✓

alice> upload image2.png
# alive_nodes = [1]
# hash(alice + image2.png) % 1 = 0
# → Node 1 processes ✓
```

Result: **System still works with single node**

### Scenario 4: Node Recovery (Back to 3)

```bash
# Node 3 comes back online

alice> upload image1.png
# alive_nodes = [1, 2, 3]  ← Node 3 detected!
# hash(alice + image1.png) % 3 = 1
# → Node 2 processes ✓
```

Result: **Load automatically redistributed when nodes recover**

## Performance Characteristics

### Latency Impact

**Per Request Overhead:**
- Health check: ~100ms (parallel for all peers)
- Hash calculation: ~1μs
- Assignment check: ~1μs
- **Total added latency: ~100ms**

**Optimization:**
- Could cache alive nodes (trade freshness for speed)
- Current approach prioritizes **correctness over speed**

### Network Impact

**Per Request:**
- Client → 3 TCP connections (broadcast)
- Each server → 2 quick connection tests (health check)
- **Total: 3 client connections + 6 health checks**

### Scalability

**Current: 3 nodes**
- Works well for small clusters
- Health checks: O(n²) per request
- Acceptable for n=3

**Future: More nodes**
- Would need caching or gossip protocol
- Current approach doesn't scale to 100+ nodes
- Good for 3-10 nodes

## Limitations

1. **100ms overhead per request** (health check time)
   - Could be optimized with caching
   - Trade-off: fresher health data vs speed

2. **Request redistribution on failure**
   - Same request may go to different server if nodes change
   - Not a problem for stateless encryption
   - Could matter for stateful systems

3. **No load-aware balancing**
   - Doesn't account for CPU/memory usage
   - Pure hash-based distribution
   - Assumes nodes have equal capacity

4. **Simultaneous failures**
   - If all nodes crash simultaneously, system is down
   - Recovery requires at least 1 node to start

## Configuration

No configuration needed! The system automatically:
- Detects number of nodes from config.toml
- Discovers alive nodes on each request
- Adjusts distribution based on availability

## Testing

### Test 1: Even Distribution
```bash
# With 3 nodes, upload 9 images
# Should see ~3 processed by each node
```

### Test 2: Node Failure
```bash
# Upload image
# Kill assigned node
# Upload same image again
# Should succeed (different node processes)
```

### Test 3: Recovery
```bash
# Kill Node 3
# Upload 6 images (distributed across Node 1, 2)
# Restart Node 3
# Upload 6 more images (distributed across all 3)
```

## Summary

This load balancing implementation provides:

✅ **Even load distribution** across all alive nodes
✅ **Automatic failover** when nodes crash
✅ **Quick recovery** when nodes restart
✅ **Deterministic assignment** (hash-based)
✅ **No race conditions** (same hash → same server)
✅ **Simple implementation** (~50 lines of code)
✅ **Fault tolerant** (works with 1-3 nodes)

Perfect for a distributed image encryption system with 3-node clusters!
