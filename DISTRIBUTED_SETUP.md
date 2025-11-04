# Distributed Deployment Guide

This guide explains how to deploy the distributed image storage system across multiple physical machines.

## Overview

The system now uses a `config.toml` file to specify server addresses, making it easy to deploy across different machines on a network.

## Quick Start (Localhost Testing)

The default configuration uses localhost addresses for testing:

```bash
# 1. Build on your machine
cargo build --release

# 2. Start servers (3 terminals)
cargo run --bin server 1
cargo run --bin server 2
cargo run --bin server 3

# 3. Start client (4th terminal)
cargo run --bin server alice
```

## Distributed Deployment Across Physical Machines

### Prerequisites

- 3 separate machines on the same network
- Rust toolchain installed on each machine
- Network connectivity between all machines
- Firewall rules allowing TCP traffic on ports 8001-8003

### Step 1: Find IP Addresses

On each machine, find its local IP address:

**Linux/Mac:**
```bash
ip addr show | grep "inet " | grep -v 127.0.0.1
# OR
ifconfig | grep "inet " | grep -v 127.0.0.1
```

**Windows:**
```cmd
ipconfig | findstr IPv4
```

Example results:
- Machine 1: `192.168.1.10`
- Machine 2: `192.168.1.11`
- Machine 3: `192.168.1.12`

### Step 2: Configure Firewall Rules

On each machine, open the required port:

**Linux (UFW):**
```bash
# On Machine 1:
sudo ufw allow 8001/tcp

# On Machine 2:
sudo ufw allow 8002/tcp

# On Machine 3:
sudo ufw allow 8003/tcp
```

**Linux (firewalld):**
```bash
# On Machine 1:
sudo firewall-cmd --permanent --add-port=8001/tcp
sudo firewall-cmd --reload

# On Machine 2:
sudo firewall-cmd --permanent --add-port=8002/tcp
sudo firewall-cmd --reload

# On Machine 3:
sudo firewall-cmd --permanent --add-port=8003/tcp
sudo firewall-cmd --reload
```

**Windows Firewall:**
```cmd
# On Machine 1:
netsh advfirewall firewall add rule name="Distinsta Server 1" dir=in action=allow protocol=TCP localport=8001

# On Machine 2:
netsh advfirewall firewall add rule name="Distinsta Server 2" dir=in action=allow protocol=TCP localport=8002

# On Machine 3:
netsh advfirewall firewall add rule name="Distinsta Server 3" dir=in action=allow protocol=TCP localport=8003
```

### Step 3: Copy Project to Each Machine

On each machine:

```bash
# Clone the repository
git clone <your-repo-url> distinsta
cd distinsta

# OR copy the project directory
scp -r distinsta user@192.168.1.10:~
scp -r distinsta user@192.168.1.11:~
scp -r distinsta user@192.168.1.12:~
```

### Step 4: Edit config.toml

On **ALL machines** (servers and client), edit `config.toml`:

```toml
[servers]
node1 = "192.168.1.10:8001"
node2 = "192.168.1.11:8002"
node3 = "192.168.1.12:8003"
```

**Important:** All machines must have the **exact same** `config.toml` file!

### Step 5: Build on Each Machine

On each of the 3 server machines:

```bash
cd distinsta
cargo build --release
```

### Step 6: Start Server Nodes

**On Machine 1 (192.168.1.10):**
```bash
cd distinsta
cargo run --bin server 1
```

**On Machine 2 (192.168.1.11):**
```bash
cd distinsta
cargo run --bin server 2
```

**On Machine 3 (192.168.1.12):**
```bash
cd distinsta
cargo run --bin server 3
```

You should see output like:
```
Node 1 will bind to 192.168.1.10:8001
Starting Server Node 1 on 192.168.1.10:8001
Node 1 listening on 192.168.1.10:8001
Node 1: Starting initial election
```

Wait 5-8 seconds for leader election to complete. You should see:
```
Node 3: I am the new leader!
Node 3: I am the LEADER, initializing load balancer
```

### Step 7: Run Client

From **any machine** on the network (can be one of the server machines or a 4th machine):

```bash
cd distinsta
cargo run --bin client alice
```

You should see:
```
Client will broadcast to servers:
  - 192.168.1.10:8001
  - 192.168.1.11:8002
  - 192.168.1.12:8003

=== Distributed Image Storage Client (REPL) ===
User: alice
Multicast mode: Broadcasting to all servers
Type 'help' for commands, 'quit' to exit
================================================

alice>
```

Now you can upload images:
```
alice> upload test_image.png
```

## Network Diagram

```
┌─────────────────────────────────────────────────┐
│  Client Machine (any machine on network)        │
│  IP: 192.168.1.X                                │
│  Broadcasts to all servers                      │
└────────────┬────────────────────────────────────┘
             │
             │ Multicast to all
             │
     ┌───────┴───────────────────────┐
     │                               │
     v                               v
┌─────────────────┐        ┌─────────────────┐
│  Machine 1      │        │  Machine 2      │
│  192.168.1.10   │◄──────►│  192.168.1.11   │
│  Node 1:8001    │        │  Node 2:8002    │
│  (Worker)       │        │  (Worker)       │
└────────┬────────┘        └────────┬────────┘
         │                          │
         │   Bully Election        │
         │   + Heartbeats          │
         │                          │
         └──────────┬───────────────┘
                    │
                    v
         ┌─────────────────┐
         │  Machine 3      │
         │  192.168.1.12   │
         │  Node 3:8003    │
         │  (Leader)       │
         └─────────────────┘
```

## Troubleshooting

### Connection Refused

**Problem:** Client shows "Connection refused"

**Solutions:**
1. Check firewall rules are correctly configured
2. Verify server is running: `ps aux | grep server`
3. Check server is listening: `netstat -tuln | grep 800[1-3]`
4. Ensure `config.toml` has correct IP addresses
5. Ping each machine to verify network connectivity

### Address Already in Use

**Problem:** Server shows "Address already in use"

**Solution:**
```bash
# Kill existing server
pkill -f "cargo run --bin server"
# OR
killall server

# Wait a few seconds, then restart
sleep 2
cargo run --bin server <node_id>
```

### Leader Election Not Completing

**Problem:** No leader elected after 10 seconds

**Solutions:**
1. Check all 3 servers are running
2. Verify network connectivity between servers
3. Check firewall isn't blocking connections
4. Ensure `config.toml` is identical on all machines

### Cannot Upload Images

**Problem:** "No server processed the request"

**Solutions:**
1. Wait 5-8 seconds after starting servers for election to complete
2. Check that at least one server shows "I am the LEADER"
3. Verify client's `config.toml` matches server configurations
4. Check network connectivity from client to all servers

## Configuration File Format

The `config.toml` file uses TOML format:

```toml
# Lines starting with # are comments

[servers]
node1 = "IP_ADDRESS:PORT"
node2 = "IP_ADDRESS:PORT"
node3 = "IP_ADDRESS:PORT"
```

**Important Notes:**
- Use quotes around addresses
- Format: `"IP:PORT"`
- All machines need identical config
- No trailing commas

## Testing the Distributed Setup

### Test 1: Basic Upload
```bash
alice> upload test_image.png
# Should see: "✓ Server X processed request"
```

### Test 2: Multiple Uploads
```bash
alice> upload test_image.png
alice> upload test_image.png
alice> upload test_image.png
# Should create 3 files with different timestamps
```

### Test 3: Failover
1. Upload an image (note which server processed it)
2. Kill Node 3 (the leader): `Ctrl+C` on Machine 3
3. Wait 5-10 seconds
4. Watch Node 2 become leader: "Node 2: I am the new leader!"
5. Upload another image - should still work with Node 2 as leader

### Test 4: Load Distribution
```bash
# Upload several images quickly
alice> upload img1.png
alice> upload img2.png
alice> upload img3.png
alice> upload img4.png
# Should see different nodes processing (round-robin via timestamp)
```

## Performance Considerations

### Network Latency
- Client broadcasts to all servers simultaneously
- Response time limited by slowest network path
- Consider using wired connections for servers

### Firewall Impact
- Each upload requires TCP connections to all 3 servers
- Firewall rules should allow established connections
- Consider disabling SPI (Stateful Packet Inspection) for better performance

### Bandwidth Usage
- Client sends full image to all servers
- Only one server processes and returns encrypted image
- For 1MB image: ~3MB upload, ~1MB download per request

## Security Considerations

This is an educational implementation. For production use:

1. **TLS/SSL**: Encrypt network traffic
2. **Authentication**: Verify client identity
3. **Authorization**: Control who can upload
4. **Key Management**: Don't derive keys from usernames
5. **Input Validation**: Validate image files
6. **Rate Limiting**: Prevent abuse
7. **Firewall**: Restrict access to known clients

## Alternative Configurations

### Single Machine Testing with Different Ports
```toml
[servers]
node1 = "127.0.0.1:8001"
node2 = "127.0.0.1:8002"
node3 = "127.0.0.1:8003"
```

### Mixed Local/Remote
```toml
[servers]
node1 = "127.0.0.1:8001"        # Local
node2 = "192.168.1.11:8002"     # Remote
node3 = "192.168.1.12:8003"     # Remote
```

### Custom Ports
```toml
[servers]
node1 = "192.168.1.10:9001"
node2 = "192.168.1.11:9002"
node3 = "192.168.1.12:9003"
```

## Summary

The configuration file approach makes distributed deployment simple:

1. Install Rust on each machine
2. Copy project to each machine
3. Find IP addresses
4. Configure firewall rules
5. Edit `config.toml` with IP addresses
6. Build and run servers
7. Run client from any machine

The system automatically handles:
- Leader election
- Load distribution
- Failover recovery
- Image encryption
- Unique filenames
