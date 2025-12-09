mod config;
mod discovery;
mod p2p_protocol;
mod protocol;
mod steganography;
mod web_gui;

use config::Config;
use discovery::{ImageInfo, UserInfo};
use p2p_protocol::{P2PRequest, P2PResponse};
use protocol::{ClientRequest, ServerResponse};
use steganography::{create_access_denied_image, embed_metadata, extract_metadata, ImageMetadata};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub struct Client {
    pub username: String,
    server_addresses: Vec<String>,
    p2p_port: u16,
    p2p_address: String,
    pub owned_images: Arc<RwLock<HashMap<String, PathBuf>>>,    // image_id -> path
    pub received_images: Arc<RwLock<HashMap<String, PathBuf>>>, // image_id -> path
}

impl Client {
    fn new(username: String, server_addresses: Vec<String>, p2p_port: u16, p2p_address: String) -> Self {
        Client {
            username,
            server_addresses,
            p2p_port,
            p2p_address,
            owned_images: Arc::new(RwLock::new(HashMap::new())),
            received_images: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn send_request(&self, request: ClientRequest) -> Result<ServerResponse, Box<dyn std::error::Error>> {
        let request_json = serde_json::to_string(&request)?;

        // Try each server until one responds successfully
        for address in &self.server_addresses {
            match TcpStream::connect(address).await {
                Ok(mut stream) => {
                    stream.write_all(request_json.as_bytes()).await?;
                    stream.write_all(b"\n").await?;

                    let mut reader = BufReader::new(&mut stream);
                    let mut response_line = String::new();
                    reader.read_line(&mut response_line).await?;

                    let response: ServerResponse = serde_json::from_str(&response_line)?;
                    return Ok(response);
                }
                Err(_) => continue,
            }
        }

        Err("All servers unavailable".into())
    }

    async fn register(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Registering with discovery service...");
        let request = ClientRequest::Register {
            username: self.username.clone(),
            p2p_address: self.p2p_address.clone(),
        };

        match self.send_request(request).await? {
            ServerResponse::Registered { success, message } => {
                if success {
                    println!("✓ Registered successfully");
                } else {
                    println!("✗ Registration failed: {}", message);
                }
                Ok(())
            }
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn get_peers(&self) -> Result<Vec<UserInfo>, Box<dyn std::error::Error>> {
        let request = ClientRequest::GetPeers {
            username: self.username.clone(),
        };

        match self.send_request(request).await? {
            ServerResponse::PeerList { peers } => Ok(peers),
            _ => Err("Unexpected response".into()),
        }
    }

    async fn publish_image(&self, image_id: String, filename: String, shared_with: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let image_info = ImageInfo {
            image_id,
            filename,
            owner: self.username.clone(),
            shared_with,
        };

        let request = ClientRequest::PublishImage { image_info };
        self.send_request(request).await?;
        println!("✓ Image published to discovery service");
        Ok(())
    }

    pub async fn share_image(&self, image_path: &str, image_id: String, username: String, views: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sharing image '{}' with {} ({} views)", image_id, username, views);

        // Load image directly from file path (better format detection)
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to load image '{}': {}", image_path, e))?
            .to_rgba8();

        // Create or update metadata
        let mut metadata = match extract_metadata(&img) {
            Ok(m) => m,
            Err(_) => ImageMetadata::new(self.username.clone(), image_id.clone()),
        };

        metadata.add_permission(username.clone(), views);

        // Embed metadata
        let embedded_img = embed_metadata(&img, &metadata)?;

        // Save to owned directory
        let owned_dir = format!("images/owned_{}", self.username);
        fs::create_dir_all(&owned_dir)?;
        let save_path = format!("{}/{}.png", owned_dir, image_id);
        embedded_img.save(&save_path)?;

        // Store reference
        let mut owned = self.owned_images.write().await;
        owned.insert(image_id.clone(), PathBuf::from(save_path));

        println!("✓ Image saved with permissions");
        Ok(())
    }

    pub async fn request_image_from_peer(&self, owner: &str, image_id: &str, owner_p2p_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Requesting image '{}' from {}", image_id, owner);

        let request = P2PRequest::RequestImage {
            requester: self.username.clone(),
            image_id: image_id.to_string(),
        };

        let mut stream = TcpStream::connect(owner_p2p_addr).await?;
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        let mut reader = BufReader::new(&mut stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;

        let response: P2PResponse = serde_json::from_str(&response_line)?;

        match response {
            P2PResponse::ImageData { image_id: recv_id, data } => {
                // Save to received directory
                let received_dir = format!("images/received_{}", self.username);
                fs::create_dir_all(&received_dir)?;
                let save_path = format!("{}/{}_{}.png", received_dir, owner, recv_id);
                fs::write(&save_path, data)?;

                let mut received = self.received_images.write().await;
                received.insert(recv_id.clone(), PathBuf::from(save_path));

                println!("✓ Image received and saved");
                Ok(())
            }
            P2PResponse::AccessDenied { reason } => {
                Err(format!("Access denied: {}", reason).into())
            }
            P2PResponse::Error { message } => {
                Err(format!("Error: {}", message).into())
            }
            _ => Err("Unexpected response".into()),
        }
    }

    async fn view_image(&self, image_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check owned images first
        let owned = self.owned_images.read().await;
        if let Some(path) = owned.get(image_id) {
            println!("Viewing owned image: {}", image_id);
            println!("Path: {}", path.display());
            return Ok(());
        }
        drop(owned);

        // Check received images
        let received = self.received_images.read().await;
        if let Some(path) = received.get(image_id) {
            let path_clone = path.clone();
            drop(received);

            // Load image and check permissions
            let img_bytes = fs::read(&path_clone)?;
            let img = image::load_from_memory(&img_bytes)?.to_rgba8();

            let mut metadata = extract_metadata(&img)?;

            if metadata.can_view(&self.username) {
                if metadata.decrement_view(&self.username) {
                    // Update image with decremented view count
                    let updated_img = embed_metadata(&img, &metadata)?;
                    updated_img.save(&path_clone)?;

                    let remaining = metadata.get_remaining_views(&self.username);
                    println!("✓ Viewing image: {}", image_id);
                    println!("  Remaining views: {}", remaining);
                    println!("  Path: {}", path_clone.display());
                    return Ok(());
                }
            }

            // Access denied - show default image
            println!("✗ Access denied or views exhausted");
            let denied_img = create_access_denied_image();
            let denied_path = format!("images/access_denied_{}.png", image_id);
            denied_img.save(&denied_path)?;
            println!("  Showing default image: {}", denied_path);
            return Ok(());
        }

        Err("Image not found".into())
    }

    async fn list_my_images(&self) {
        let owned = self.owned_images.read().await;
        println!("\n=== My Images ===");
        if owned.is_empty() {
            println!("No images owned");
        } else {
            for (id, path) in owned.iter() {
                println!("  - {} ({})", id, path.display());
            }
        }
    }

    async fn list_received_images(&self) {
        let received = self.received_images.read().await;
        println!("\n=== Received Images ===");
        if received.is_empty() {
            println!("No images received");
        } else {
            for (id, path) in received.iter() {
                println!("  - {} ({})", id, path.display());
            }
        }
    }

    async fn start_p2p_server(self: Arc<Self>) {
        let bind_addr = format!("0.0.0.0:{}", self.p2p_port);
        let listener = TcpListener::bind(&bind_addr).await.unwrap();
        println!("P2P server listening on {}", bind_addr);
        println!("P2P address registered as: {}", self.p2p_address);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("P2P connection from {}", addr);
                    let client = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = client.handle_p2p_request(stream).await {
                            eprintln!("P2P request error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("P2P accept error: {}", e);
                }
            }
        }
    }

    async fn handle_p2p_request(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let request: P2PRequest = serde_json::from_str(&line)?;

        let response = match request {
            P2PRequest::RequestImage { requester, image_id } => {
                // Check if I own this image
                let owned = self.owned_images.read().await;
                if let Some(path) = owned.get(&image_id) {
                    let path_clone = path.clone();
                    drop(owned);

                    // Load image and check permissions
                    let img_bytes = fs::read(&path_clone)?;
                    let img = image::load_from_memory(&img_bytes)?.to_rgba8();

                    match extract_metadata(&img) {
                        Ok(metadata) => {
                            if metadata.can_view(&requester) {
                                P2PResponse::ImageData {
                                    image_id,
                                    data: img_bytes,
                                }
                            } else {
                                P2PResponse::AccessDenied {
                                    reason: "No permission or views exhausted".to_string(),
                                }
                            }
                        }
                        Err(e) => P2PResponse::Error {
                            message: format!("Metadata error: {}", e),
                        },
                    }
                } else {
                    P2PResponse::Error {
                        message: "Image not found".to_string(),
                    }
                }
            }
            P2PRequest::GetImageList { requester: _ } => {
                let owned = self.owned_images.read().await;
                let images: Vec<ImageInfo> = owned
                    .keys()
                    .map(|id| ImageInfo {
                        image_id: id.clone(),
                        filename: id.clone(),
                        owner: self.username.clone(),
                        shared_with: vec![],
                    })
                    .collect();
                P2PResponse::ImageList { images }
            }
        };

        let response_json = serde_json::to_string(&response)?;
        stream.write_all(response_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        Ok(())
    }

    async fn heartbeat_loop(self: Arc<Self>) {
        loop {
            sleep(Duration::from_secs(30)).await;
            let request = ClientRequest::Heartbeat {
                username: self.username.clone(),
            };
            let _ = self.send_request(request).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <username> <p2p_port> [ip_address]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  Local testing:    cargo run --bin client alice 9001");
        eprintln!("  Distributed:      cargo run --bin client alice 9001 192.168.1.100");
        eprintln!();
        eprintln!("To find your IP:");
        eprintln!("  Linux/Mac:  hostname -I");
        eprintln!("  Windows:    ipconfig");
        std::process::exit(1);
    }

    let username = args[1].clone();
    let p2p_port: u16 = args[2].parse().expect("Invalid port");

    let config = Config::load("config.toml").expect("Failed to load config");
    let server_addresses: Vec<String> = config.servers.values().cloned().collect();

    // Determine P2P address for registration
    // User can provide IP as 3rd argument, otherwise use 0.0.0.0 (localhost only)
    let p2p_ip = if args.len() > 3 {
        args[3].clone()  // User provides their IP: cargo run --bin client alice 9001 192.168.1.100
    } else {
        "127.0.0.1".to_string()  // Default to localhost for local testing
    };
    let p2p_register_address = format!("{}:{}", p2p_ip, p2p_port);

    let client = Arc::new(Client::new(username.clone(), server_addresses, p2p_port, p2p_register_address));

    // Start P2P server
    let client_clone = Arc::clone(&client);
    tokio::spawn(async move {
        client_clone.start_p2p_server().await;
    });

    // Start web GUI server
    let client_clone = Arc::clone(&client);
    tokio::spawn(async move {
        web_gui::start_web_server(client_clone).await;
    });

    // Wait for P2P server to start
    sleep(Duration::from_secs(1)).await;

    // Register with discovery service
    if let Err(e) = client.register().await {
        eprintln!("Registration failed: {}", e);
    }

    // Start heartbeat
    let client_clone = Arc::clone(&client);
    tokio::spawn(async move {
        client_clone.heartbeat_loop().await;
    });

    // Command loop
    println!("\n=== Distinsta P2P Image Sharing Client ===");
    println!("User: {}", username);
    println!("P2P Port: {}", p2p_port);
    println!("\nCommands:");
    println!("  peers                                    - List online peers");
    println!("  share <image_path> <image_id> <user> <views> - Share image with peer");
    println!("  request <owner> <image_id> <owner_p2p_addr>  - Request image from peer");
    println!("  view <image_id>                          - View an image");
    println!("  my_images                                - List my images");
    println!("  received                                 - List received images");
    println!("  help                                     - Show commands");
    println!("  quit                                     - Exit");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        if reader.read_line(&mut input).await.is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "peers" => {
                match client.get_peers().await {
                    Ok(peers) => {
                        println!("\n=== Online Peers ===");
                        if peers.is_empty() {
                            println!("No peers online");
                        } else {
                            for peer in peers {
                                println!("  - {} ({})", peer.username, peer.p2p_address);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "share" => {
                if parts.len() != 5 {
                    println!("Usage: share <image_path> <image_id> <user> <views>");
                    continue;
                }
                let image_path = parts[1];
                let image_id = parts[2].to_string();
                let target_user = parts[3].to_string();
                let views: u32 = parts[4].parse().unwrap_or(0);

                match client.share_image(image_path, image_id.clone(), target_user.clone(), views).await {
                    Ok(_) => {
                        // Publish to discovery
                        let _ = client.publish_image(image_id, image_path.to_string(), vec![target_user]).await;
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "request" => {
                if parts.len() != 4 {
                    println!("Usage: request <owner> <image_id> <owner_p2p_addr>");
                    continue;
                }
                let owner = parts[1];
                let image_id = parts[2];
                let owner_addr = parts[3];

                match client.request_image_from_peer(owner, image_id, owner_addr).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "view" => {
                if parts.len() != 2 {
                    println!("Usage: view <image_id>");
                    continue;
                }
                let image_id = parts[1];
                match client.view_image(image_id).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            "my_images" => {
                client.list_my_images().await;
            }
            "received" => {
                client.list_received_images().await;
            }
            "help" => {
                println!("\nCommands:");
                println!("  peers                                    - List online peers");
                println!("  share <image_path> <image_id> <user> <views> - Share image with peer");
                println!("  request <owner> <image_id> <owner_p2p_addr>  - Request image from peer");
                println!("  view <image_id>                          - View an image");
                println!("  my_images                                - List my images");
                println!("  received                                 - List received images");
                println!("  quit                                     - Exit");
                println!("\nP2P Addressing:");
                println!("  The 'peers' command shows each peer's P2P address.");
                println!("  Use that EXACT address in the 'request' command.");
                println!("  Local: 127.0.0.1:port  |  Distributed: actual_ip:port");
            }
            "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command. Type 'help' for commands.");
            }
        }
    }
}
