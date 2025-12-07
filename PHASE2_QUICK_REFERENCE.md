# Phase 2 Quick Reference

## Starting the System

### Servers (3 terminals)
```bash
cargo run --bin server 1    # Node 1
cargo run --bin server 2    # Node 2
cargo run --bin server 3    # Node 3
```

### Clients (2+ terminals)
```bash
cargo run --bin client alice 9001
cargo run --bin client bob 9002
cargo run --bin client charlie 9003
```

## Client Commands

```
peers                                           List online users
share <image_path> <image_id> <user> <views>   Share with permissions
request <owner> <image_id> <p2p_addr>          Get image from peer
view <image_id>                                 View (decrements count)
my_images                                       List owned images
received                                        List received images
help                                            Show commands
quit                                            Exit
```

## Example Workflow

### Alice shares with Bob
```bash
# Alice
> peers                                      # See bob online
> share test_images/test1.png img001 bob 5  # Give bob 5 views

# Bob
> peers                                      # See alice online
> request alice img001 127.0.0.1:9001       # Get image
> view img001                                # View it (4 left)
> view img001                                # View it (3 left)
> view img001                                # View it (2 left)
> view img001                                # View it (1 left)
> view img001                                # View it (0 left)
> view img001                                # ACCESS DENIED!
```

## File Locations

```
images/owned_<user>/        Owned images with metadata
images/received_<user>/     Images from peers
images/access_denied_*.png  Shown when no views left
test_images/                Sample images for testing
```

## Key Features

✅ Discovery service - find online peers
✅ P2P transfer - direct client-to-client
✅ Steganography - permissions in image
✅ View limits - automatic enforcement
✅ Fault tolerance - works with 1+ servers

## Troubleshooting

**"All servers unavailable"**
→ Start at least one server first

**"Access denied"**
→ Views exhausted or no permission

**"Connection refused" for P2P**
→ Check peer's P2P address/port

**Peer not showing in list**
→ Wait 30s for heartbeat or re-register

## Architecture

```
Cloud Servers (Discovery Service)
    ↓ register, get_peers
Clients (P2P Servers)
    ↔ direct image transfer (no cloud)
```

## Build Commands

```bash
cargo build                  # Build both
cargo build --bin server     # Server only
cargo build --bin client     # Client only
./test_phase2.sh            # Quick test setup
```

## For Distributed Testing

1. Edit `config.toml` - set real IPs
2. Start servers on 3 machines
3. Start clients on different machines
4. Use actual IP:PORT in request command

Example:
```bash
> request alice img001 10.40.45.206:9001
```

## Testing Checklist

- [ ] 3 servers running
- [ ] 2+ clients registered
- [ ] Can see peers
- [ ] Can share image
- [ ] Can request image
- [ ] Can view image
- [ ] View count decrements
- [ ] Access denied after quota
- [ ] Server failure handled
- [ ] State synced across servers

---
*All features implemented and working!*
