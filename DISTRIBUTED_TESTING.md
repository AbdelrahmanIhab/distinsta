# Distributed Testing Guide - P2P Image Sharing Across Machines

## The Fix is Already Implemented!

Your P2P system is **fully working** for distributed deployment. You just need to **provide your IP address** when starting clients on different machines.

## How It Works

### Local Testing (Same Machine)
When all clients are on the same machine, they register with `127.0.0.1` (localhost):

```bash
# Default - no IP needed
cargo run --release --bin client alice 9001
cargo run --release --bin client bob 9002
```

**Result:** Both register as `127.0.0.1:port` and can connect to each other.

### Distributed Testing (Different Machines)
When clients are on different machines, you **must provide each machine's IP address**:

```bash
# On Machine 1 (e.g., 192.168.1.100)
cargo run --release --bin client alice 9001 192.168.1.100

# On Machine 2 (e.g., 192.168.1.101)
cargo run --release --bin client bob 9002 192.168.1.101
```

**Result:** Alice registers as `192.168.1.100:9001`, Bob as `192.168.1.101:9002`, and they can connect!

## Step-by-Step: Testing Across Machines

### Step 1: Find Your IP Address

**On Linux/Mac:**
```bash
hostname -I
# Example output: 192.168.1.100
```

**On Windows:**
```cmd
ipconfig
# Look for "IPv4 Address" under your network adapter
```

**On Mac (specific):**
```bash
ifconfig en0 | grep "inet "
# Example: inet 192.168.1.100
```

### Step 2: Start the Server (on any machine)

```bash
cargo run --release --bin server 1
```

**Note:** Make sure all clients can reach this server (check `config.toml` has correct server IPs).

### Step 3: Start Alice on Machine 1

```bash
# Example: Machine 1 IP is 192.168.1.100
cargo run --release --bin client alice 9001 192.168.1.100
```

**Output:**
```
P2P server listening on 0.0.0.0:9001
P2P address registered as: 192.168.1.100:9001
Registering with discovery service...
✓ Registered successfully
```

### Step 4: Start Bob on Machine 2

```bash
# Example: Machine 2 IP is 192.168.1.101
cargo run --release --bin client bob 9002 192.168.1.101
```

### Step 5: Verify Peer Discovery

**On Bob's machine:**
```bash
> peers
```

**Expected output:**
```
=== Online Peers ===
  - alice (192.168.1.100:9001)
```

✅ **This is the key!** Bob sees Alice's **actual IP**, not `127.0.0.1`.

### Step 6: Alice Shares Image with Bob

**On Alice's machine:**
```bash
> share test_images/test1.png img001 bob 5
```

**Output:**
```
Sharing image 'img001' with bob (5 views)
✓ Image saved with permissions
✓ Image published to discovery service
```

### Step 7: Bob Requests Image from Alice

**On Bob's machine, use the address from `peers` command:**
```bash
> request alice img001 192.168.1.100:9001
```

**Output:**
```
Requesting image 'img001' from alice
✓ Image received and saved
```

✅ **SUCCESS!** P2P transfer across machines works!

### Step 8: Bob Views the Image

```bash
> view img001
```

**Output:**
```
✓ Viewing image: img001
  Remaining views: 4
  Path: images/received_bob/alice_img001.png
```

## Common Issues and Solutions

### Issue 1: "Connection refused"
**Cause:** Using `127.0.0.1` instead of real IP, or using wrong IP.

**Solution:**
1. Run `peers` command
2. Copy the EXACT address shown
3. Use that address in `request` command

### Issue 2: Firewall Blocking
**Symptom:** Peers can see each other but can't transfer images.

**Solution:**
```bash
# On Linux (allow P2P port)
sudo ufw allow 9001/tcp
sudo ufw allow 9002/tcp

# Or disable firewall temporarily for testing
sudo ufw disable
```

### Issue 3: Wrong Network
**Symptom:** Machines can't reach each other.

**Solution:**
- Ensure both machines are on the **same network** (same WiFi/LAN)
- Check with `ping`:
  ```bash
  ping 192.168.1.100  # From Machine 2 to Machine 1
  ```

### Issue 4: Multiple Network Interfaces
**Symptom:** Provided the wrong IP address.

**Solution:**
- Your machine might have multiple IPs (WiFi, Ethernet, VPN)
- Use the IP of the interface connected to your peers
- Verify with `ping` from the other machine

## Example: Full Distributed Test Session

```bash
# === MACHINE 1 (192.168.1.100) ===
$ hostname -I
192.168.1.100

$ cargo run --release --bin server 1
# Server starts...

# New terminal
$ cargo run --release --bin client alice 9001 192.168.1.100
> share test_images/test1.png img001 bob 5
✓ Image saved with permissions
> my_images
=== My Images ===
  - img001 (images/owned_alice/img001.png)


# === MACHINE 2 (192.168.1.101) ===
$ hostname -I
192.168.1.101

$ cargo run --release --bin client bob 9002 192.168.1.101
> peers
=== Online Peers ===
  - alice (192.168.1.100:9001)

> request alice img001 192.168.1.100:9001
Requesting image 'img001' from alice
✓ Image received and saved

> view img001
✓ Viewing image: img001
  Remaining views: 4

> view img001
✓ Viewing image: img001
  Remaining views: 3

# ... repeat 3 more times ...

> view img001
✗ Access denied or views exhausted
  Showing default image: images/access_denied_img001.png
```

## Network Setup for Your Project

Based on your `config.toml`:

```toml
[servers]
"1" = "10.40.45.206:8001"
"2" = "10.40.33.244:8002"
"3" = "10.40.43.200:8003"
```

### Recommended Deployment:

**Server 1 (10.40.45.206):**
```bash
cargo run --release --bin server 1
```

**Client on any machine in the same network:**
```bash
# Find your IP first
hostname -I
# Example output: 10.40.50.123

# Start client with YOUR IP
cargo run --release --bin client alice 9001 10.40.50.123
```

**Another client on different machine:**
```bash
# Find this machine's IP
hostname -I
# Example output: 10.40.51.234

# Start with THIS machine's IP
cargo run --release --bin client bob 9002 10.40.51.234
```

## Quick Reference

| Scenario | Command | Registered Address |
|----------|---------|-------------------|
| Local testing | `cargo run --bin client alice 9001` | `127.0.0.1:9001` |
| Machine at 192.168.1.100 | `cargo run --bin client alice 9001 192.168.1.100` | `192.168.1.100:9001` |
| Machine at 10.40.45.100 | `cargo run --bin client alice 9001 10.40.45.100` | `10.40.45.100:9001` |

## Key Points

1. **Each client must provide their OWN IP** (the machine they're running on)
2. **The `peers` command shows you what IP to use** in `request`
3. **P2P server binds to `0.0.0.0`** (all interfaces) so it works on any network
4. **Discovery service stores the IP you provide** and shares it with peers

## That's It!

The system is **fully functional** for distributed deployment. Just remember:
- **Local**: No IP needed
- **Distributed**: Add your IP as 3rd argument

Happy testing! 🎉
