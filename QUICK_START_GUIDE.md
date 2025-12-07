# Quick Start Guide - P2P Image Sharing

## Step-by-Step: Share an Image Between Two Clients

### Prerequisites
Make sure you have test images:
```bash
ls test_images/
# Should show: test1.png, test2.png
```

### Step 1: Start the Server
Open **Terminal 1**:
```bash
cargo run --bin server 1
```

Wait until you see:
```
Starting Server Node 1 on 10.40.45.206:8001
Node 1 listening on 10.40.45.206:8001
Node 1: I am the LEADER
```

### Step 2: Start Alice's Client
Open **Terminal 2**:
```bash
cargo run --bin client alice 9001
```

You should see:
```
P2P server listening on 0.0.0.0:9001
Registering with discovery service...
✓ Registered successfully

=== Distinsta P2P Image Sharing Client ===
User: alice
P2P Port: 9001

>
```

### Step 3: Start Bob's Client
Open **Terminal 3**:
```bash
cargo run --bin client bob 9002
```

Same output as Alice but with different port.

### Step 4: Alice Shares Image with Bob

In **Alice's terminal** (Terminal 2):
```bash
> share test_images/test1.png img001 bob 5
```

Expected output:
```
Sharing image 'img001' with bob (5 views)
✓ Image saved with permissions
✓ Image published to discovery service
```

This creates: `images/owned_alice/img001.png` with Bob's permissions embedded.

### Step 5: Bob Checks Online Peers

In **Bob's terminal** (Terminal 3):
```bash
> peers
```

Expected output:
```
=== Online Peers ===
  - alice (0.0.0.0:9001)
```

### Step 6: Bob Requests Image from Alice

In **Bob's terminal**:
```bash
> request alice img001 127.0.0.1:9001
```

**Note**: Use `127.0.0.1:9001` for local testing, or the actual IP if on different machines.

Expected output:
```
Requesting image 'img001' from alice
✓ Image received and saved
```

This saves to: `images/received_bob/alice_img001.png`

### Step 7: Bob Views the Image

In **Bob's terminal**:
```bash
> view img001
```

Expected output (first view):
```
✓ Viewing image: img001
  Remaining views: 4
  Path: images/received_bob/alice_img001.png
```

Each time Bob runs `view img001`, the counter decrements.

### Step 8: Bob Exhausts His Views

Keep viewing:
```bash
> view img001   # 3 remaining
> view img001   # 2 remaining
> view img001   # 1 remaining
> view img001   # 0 remaining
> view img001   # ACCESS DENIED!
```

On the last view (after 5 total views):
```
✗ Access denied or views exhausted
  Showing default image: images/access_denied_img001.png
```

## Common Issues and Solutions

### Issue 1: "The image format could not be determined"
**Solution**: Make sure test images exist:
```bash
python3 -c "
from PIL import Image
img = Image.new('RGB', (400, 300), color=(73, 109, 137))
img.save('test_images/test1.png')
"
```

### Issue 2: "All servers unavailable"
**Solution**: Start at least one server first before starting clients.

### Issue 3: "Connection refused" when requesting image
**Possible causes**:
1. Wrong P2P address - use `127.0.0.1:9001` for local testing
2. Owner's client not running
3. Firewall blocking the port

**Fix**: Make sure both clients are running and use the correct address.

### Issue 4: Peer not showing in list
**Solution**: Wait 30 seconds for heartbeat, or restart the client to re-register.

### Issue 5: "No such file or directory" when sharing
**Solution**: Use correct path relative to project root:
```bash
> share test_images/test1.png img001 bob 5
```
NOT:
```bash
> share test1.png img001 bob 5   # Wrong - file not in current dir
```

## Advanced Usage

### Multiple Peers
Alice can share with multiple users:
```bash
# Alice's terminal
> share test_images/test1.png img001 bob 5
> share test_images/test2.png img002 charlie 3
```

### Check Your Images
```bash
> my_images      # Show images you own
> received       # Show images you received from others
```

### Testing on Multiple Machines

1. **Edit config.toml** - Set actual machine IPs:
```toml
[servers]
"1" = "192.168.1.10:8001"
"2" = "192.168.1.11:8002"
"3" = "192.168.1.12:8003"
```

2. **Get your IP** on the client machine:
```bash
hostname -I
# Example: 192.168.1.20
```

3. **Start client with actual IP** (for P2P):
```bash
# On machine 192.168.1.20
cargo run --bin client alice 9001
# P2P will listen on 0.0.0.0:9001
```

4. **Request using actual IP**:
```bash
# Bob requests from Alice at 192.168.1.20
> request alice img001 192.168.1.20:9001
```

## File Structure After Testing

```
distinsta/
├── images/
│   ├── owned_alice/
│   │   └── img001.png              ← Alice's copy with metadata
│   ├── received_bob/
│   │   └── alice_img001.png        ← Bob's received copy
│   └── access_denied_img001.png    ← Generated when Bob exhausts views
└── test_images/
    ├── test1.png                   ← Original test image
    └── test2.png
```

## Complete Example Session

```bash
# Terminal 1 - Server
$ cargo run --bin server 1
[Server starts and becomes leader]

# Terminal 2 - Alice
$ cargo run --bin client alice 9001
> share test_images/test1.png img001 bob 5
✓ Image saved with permissions
> my_images
=== My Images ===
  - img001 (images/owned_alice/img001.png)

# Terminal 3 - Bob
$ cargo run --bin client bob 9002
> peers
=== Online Peers ===
  - alice (0.0.0.0:9001)
> request alice img001 127.0.0.1:9001
✓ Image received and saved
> view img001
✓ Viewing image: img001
  Remaining views: 4
> received
=== Received Images ===
  - img001 (images/received_bob/alice_img001.png)
```

## Troubleshooting Tips

1. **Build first**: `cargo build` before running
2. **Check ports**: Make sure ports 8001, 9001, 9002 are not in use
3. **File permissions**: Ensure `images/` directory is writable
4. **Path issues**: Always run from project root (`/home/abdelrahman/distinsta`)
5. **Clear old data**: Remove `images/` directory if you want fresh start

## What's Happening Behind the Scenes

1. **share**: Alice embeds Bob's permissions in the image using steganography
2. **request**: Bob connects to Alice's P2P server, Alice checks permissions, sends image
3. **view**: Bob's client extracts metadata, checks quota, decrements counter, re-saves image
4. **Discovery**: Cloud server tracks who's online and can be contacted

---

**You're ready to test!** Try the complete workflow above. If you get stuck, check the error messages - they now provide detailed information about what went wrong.
