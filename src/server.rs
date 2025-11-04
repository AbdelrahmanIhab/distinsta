mod bully;
mod config;
mod encryption;
mod loadbalancer;
mod protocol;

use bully::{BullyElection, BullyMessage};
use config::Config;
use encryption::{encrypt_data, generate_key_from_username};
use loadbalancer::LoadBalancer;
use protocol::{ClientRequest, ServerResponse};
use std::env;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};

struct ServerNode {
    id: u32,
    address: String,
    bully: Arc<BullyElection>,
    load_balancer: Option<LoadBalancer>,
}

impl ServerNode {
    fn new(id: u32, address: String) -> Self {
        let bully = Arc::new(BullyElection::new(id, address.clone()));

        ServerNode {
            id,
            address: address.clone(),
            bully,
            load_balancer: None,
        }
    }

    async fn add_peer(&self, peer_id: u32, peer_address: String) {
        self.bully.add_peer(peer_id, peer_address).await;
    }

    async fn start(&mut self) {
        println!("Starting Server Node {} on {}", self.id, self.address);

        // Start listening
        let listener = TcpListener::bind(&self.address).await.unwrap();
        println!("Node {} listening on {}", self.id, self.address);

        // Wait a bit for all nodes to start
        sleep(Duration::from_secs(2)).await;

        // Start election
        println!("Node {}: Starting initial election", self.id);
        self.bully.start_election().await;

        // Wait for election to complete
        sleep(Duration::from_secs(3)).await;

        // Start leader monitoring (heartbeat)
        let bully_clone = Arc::clone(&self.bully);
        bully_clone.start_leader_monitoring().await;

        // Check if I'm the leader
        if self.bully.is_leader().await {
            println!("Node {}: I am the LEADER, initializing load balancer", self.id);
            self.load_balancer = Some(LoadBalancer::new());
        } else {
            if let Some(leader_id) = self.bully.get_leader().await {
                println!("Node {}: I am a WORKER, leader is Node {}", self.id, leader_id);
            }
        }

        // Handle connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("Node {}: New connection from {}", self.id, addr);
                    let node = self.clone_for_task();
                    tokio::spawn(async move {
                        node.handle_connection(stream).await;
                    });
                }
                Err(e) => {
                    eprintln!("Node {}: Error accepting connection: {}", self.id, e);
                }
            }
        }
    }

    fn clone_for_task(&self) -> ServerNode {
        ServerNode {
            id: self.id,
            address: self.address.clone(),
            bully: Arc::clone(&self.bully),
            load_balancer: self.load_balancer.clone(),
        }
    }

    async fn handle_connection(&self, mut stream: TcpStream) {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => return,
            Ok(_) => {
                // Try to parse as BullyMessage first
                if let Ok(msg) = serde_json::from_str::<BullyMessage>(&line) {
                    if let Some(response) = self.bully.handle_message(msg).await {
                        let response_json = serde_json::to_string(&response).unwrap();
                        let _ = stream.write_all(response_json.as_bytes()).await;
                        let _ = stream.write_all(b"\n").await;
                    }
                    return;
                }

                // Try to parse as ClientRequest
                if let Ok(request) = serde_json::from_str::<ClientRequest>(&line) {
                    let response = self.handle_client_request(request).await;
                    let response_json = serde_json::to_string(&response).unwrap();
                    let _ = stream.write_all(response_json.as_bytes()).await;
                    let _ = stream.write_all(b"\n").await;
                    return;
                }

                println!("Node {}: Unknown message format", self.id);
            }
            Err(e) => {
                eprintln!("Node {}: Error reading from stream: {}", self.id, e);
            }
        }
    }

    async fn handle_client_request(&self, request: ClientRequest) -> ServerResponse {
        println!("Node {}: Received client request", self.id);

        match request {
            ClientRequest::UploadImage {
                username,
                image_data,
                filename,
            } => {
                // Create a deterministic hash for this request (username + filename)
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                username.hash(&mut hasher);
                filename.hash(&mut hasher);
                let request_hash = hasher.finish();

                // Check which peers are alive
                let alive_nodes = self.get_alive_nodes().await;

                if alive_nodes.is_empty() {
                    println!("Node {}: No alive nodes detected, processing as fallback", self.id);
                    // Process anyway as last resort
                } else {
                    // Round-robin assignment based on request hash
                    let assigned_index = (request_hash % alive_nodes.len() as u64) as usize;
                    let assigned_node_id = alive_nodes[assigned_index];

                    if assigned_node_id != self.id {
                        println!("Node {}: Request assigned to Node {} (round-robin), rejecting",
                            self.id, assigned_node_id);
                        return ServerResponse::Error {
                            message: format!("Request assigned to Node {}", assigned_node_id),
                        };
                    }

                    println!("Node {}: Assigned to me via load balancing (alive nodes: {:?})",
                        self.id, alive_nodes);
                }

                // Process the request
                println!("Node {}: Processing image upload for user {} ({})",
                    self.id, username, filename);

                // Generate encryption key from username
                let key = generate_key_from_username(&username);

                // Encrypt the image data
                let encrypted_data = encrypt_data(&image_data, &key);

                println!("Node {}: Image encrypted ({} bytes -> {} bytes)",
                    self.id, image_data.len(), encrypted_data.len());

                // Return encrypted image to client
                ServerResponse::EncryptedImageData { data: encrypted_data }
            }
        }
    }

    /// Check which peer nodes are alive by attempting to connect
    async fn get_alive_nodes(&self) -> Vec<u32> {
        let peers = self.bully.get_all_peers().await;
        let mut alive = vec![];

        // Always include myself if I can process requests
        alive.push(self.id);

        // Quick health check for each peer
        for (peer_id, peer_addr) in peers {
            if peer_id == self.id {
                continue;
            }

            // Try to connect with short timeout
            match tokio::time::timeout(
                Duration::from_millis(100),
                TcpStream::connect(&peer_addr)
            ).await {
                Ok(Ok(_)) => {
                    alive.push(peer_id);
                }
                _ => {
                    // Node is down or unreachable
                }
            }
        }

        alive.sort();
        alive
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        LoadBalancer {
            servers: Arc::clone(&self.servers),
            next_index: Arc::clone(&self.next_index),
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <node_id>", args[0]);
        eprintln!("Example: {} 1", args[0]);
        std::process::exit(1);
    }

    let node_id: u32 = args[1].parse().expect("Node ID must be a number");

    // Load configuration from config.toml
    let config = Config::load("config.toml").expect("Failed to load config.toml");

    // Get this node's address from config
    let address = config
        .get_server_address(node_id)
        .expect(&format!("Node {} not found in config.toml", node_id));

    println!("Node {} will bind to {}", node_id, address);

    let mut node = ServerNode::new(node_id, address);

    // Add peers from config
    for peer_id in 1..=3 {
        if peer_id != node_id {
            if let Some(peer_address) = config.get_server_address(peer_id) {
                node.add_peer(peer_id, peer_address).await;
            }
        }
    }

    node.start().await;
}
