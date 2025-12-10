use std::net::IpAddr;
use tokio::net::TcpStream;

/// Detect the local IP address by connecting to one of the discovery servers
///
/// This function attempts to connect to each discovery server and inspects
/// the local socket address to determine which network interface was used.
/// This reveals the IP address that other machines can use to reach us.
pub async fn detect_local_ip(server_addresses: &[String]) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Try each server until we successfully connect
    for server_addr in server_addresses {
        match TcpStream::connect(server_addr).await {
            Ok(stream) => {
                // Get the local address from the connected socket
                if let Ok(local_addr) = stream.local_addr() {
                    let ip = local_addr.ip();

                    // Only return non-loopback addresses
                    // Loopback (127.0.0.1) won't work for distributed P2P
                    if !ip.is_loopback() {
                        return Ok(ip);
                    }
                }
            }
            Err(_) => continue, // Try next server
        }
    }

    Err("Could not detect IP: no servers reachable".into())
}

/// Format IP and port into P2P address string
pub fn format_p2p_address(ip: IpAddr, port: u16) -> String {
    format!("{}:{}", ip, port)
}
