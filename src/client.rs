mod config;
mod discovery;
mod network_utils;
mod p2p_protocol;
mod protocol;
mod steganography;
mod web_gui;

use config::Config;
use discovery::{ImageInfo, UserInfo};
use network_utils::{detect_local_ip, format_p2p_address};
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
use serde::{Deserialize, Serialize};

// Request for viewing an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewRequest {
    pub requester: String,
    pub image_id: String,
    pub requested_views: u32,
    pub requester_p2p_address: String,
}

// Info about a viewable image with remaining views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewableImageInfo {
    pub image_id: String,
    pub owner: String,
    pub remaining_views: u32,
    pub path: String,
}

pub struct Client {
    pub username: String,
    server_addresses: Vec<String>,
    p2p_port: u16,
    p2p_address: String,
    pub owned_images: Arc<RwLock<HashMap<String, PathBuf>>>,    // image_id -> path
    pub received_images: Arc<RwLock<HashMap<String, PathBuf>>>, // image_id -> path
    pub pending_requests: Arc<RwLock<Vec<ViewRequest>>>,        // Incoming requests for my images
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
            pending_requests: Arc::new(RwLock::new(Vec::new())),
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
                    println!("  P2P Address: {}", self.p2p_address);
                    println!("  Other peers can reach you at this address");
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

    pub async fn get_all_images(&self) -> Result<Vec<ImageInfo>, Box<dyn std::error::Error>> {
        let request = ClientRequest::GetAllImages {
            requester: self.username.clone(),
        };

        match self.send_request(request).await? {
            ServerResponse::ImageList { images } => Ok(images),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn get_other_users_images(&self) -> Result<Vec<ImageInfo>, Box<dyn std::error::Error>> {
        let request = ClientRequest::GetAllImages {
            requester: self.username.clone(),
        };

        match self.send_request(request).await? {
            ServerResponse::ImageList { images } => {
                // Filter out images owned by the current user
                let filtered: Vec<ImageInfo> = images
                    .into_iter()
                    .filter(|img| img.owner != self.username)
                    .collect();
                Ok(filtered)
            }
            _ => Err("Unexpected response".into()),
        }
    }

    /// Request a thumbnail from a peer
    pub async fn request_thumbnail(&self, owner_p2p_addr: &str, image_id: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let request = P2PRequest::RequestThumbnail {
            requester: self.username.clone(),
            image_id: image_id.clone(),
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
            P2PResponse::ThumbnailData { image_id: _, data } => {
                Ok(data)
            }
            P2PResponse::Error { message } => {
                Err(format!("Error: {}", message).into())
            }
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn publish_image(&self, image_id: String, filename: String, shared_with: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    /// Upload an image without targeting a specific user
    /// Makes the image available for all peers to browse and request access
    pub async fn upload_image(&self, image_path: &str, image_id: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("Uploading image '{}' from '{}'", image_id, image_path);

        // Load image directly from file path
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to load image '{}': {}", image_path, e))?
            .to_rgba8();

        // Create new metadata with owner info (no permissions yet - those come from approvals)
        let metadata = ImageMetadata::new(self.username.clone(), image_id.clone());

        // Embed metadata
        let embedded_img = embed_metadata(&img, &metadata)?;

        // Save to owned directory
        let owned_dir = format!("images/owned_{}", self.username);
        fs::create_dir_all(&owned_dir)?;
        let save_path = format!("{}/{}.png", owned_dir, image_id);
        embedded_img.save(&save_path)?;

        // Store reference
        let mut owned = self.owned_images.write().await;
        owned.insert(image_id.clone(), PathBuf::from(save_path.clone()));

        // Publish to discovery service with empty shared_with list (visible to all for browsing)
        self.publish_image(image_id.clone(), save_path, vec![]).await?;

        println!("✓ Image uploaded and published");
        Ok(())
    }

    /// Request access to view an image owned by another peer
    pub async fn request_access(&self, owner_p2p_addr: &str, image_id: String, requested_views: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("Requesting access to image '{}' ({} views)", image_id, requested_views);

        let request = P2PRequest::RequestAccess {
            requester: self.username.clone(),
            requester_p2p_address: self.p2p_address.clone(),
            image_id: image_id.clone(),
            requested_views,
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
            P2PResponse::RequestReceived { message } => {
                println!("✓ {}", message);
                Ok(())
            }
            P2PResponse::Error { message } => {
                Err(format!("Error: {}", message).into())
            }
            _ => Err("Unexpected response".into()),
        }
    }

    /// Approve a view request and grant access to the requester
    pub async fn approve_request(&self, requester: String, requester_p2p_addr: String, image_id: String, granted_views: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("Approving request from {} for image '{}' ({} views)", requester, image_id, granted_views);

        // Add permission to the image metadata
        self.share_image(
            &format!("images/owned_{}/{}.png", self.username, image_id),
            image_id.clone(),
            requester.clone(),
            granted_views
        ).await?;

        // Send approval to requester
        let request = P2PRequest::ApproveRequest {
            requester: requester.clone(),
            image_id: image_id.clone(),
            granted_views,
            owner: self.username.clone(),
            owner_p2p_address: self.p2p_address.clone(),
        };

        let mut stream = TcpStream::connect(&requester_p2p_addr).await?;
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        let mut reader = BufReader::new(&mut stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;

        let response: P2PResponse = serde_json::from_str(&response_line)?;

        match response {
            P2PResponse::ApprovalSent { message } => {
                println!("✓ {}", message);

                // Remove from pending requests
                let mut pending = self.pending_requests.write().await;
                pending.retain(|req| !(req.requester == requester && req.image_id == image_id));

                Ok(())
            }
            P2PResponse::Error { message } => {
                Err(format!("Error: {}", message).into())
            }
            _ => Err("Unexpected response".into()),
        }
    }

    /// Reject a view request
    pub async fn reject_request(&self, requester: String, image_id: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("Rejecting request from {} for image '{}'", requester, image_id);

        // Remove from pending requests
        let mut pending = self.pending_requests.write().await;
        pending.retain(|req| !(req.requester == requester && req.image_id == image_id));

        println!("✓ Request rejected");
        Ok(())
    }

    /// Get list of pending view requests
    pub async fn get_pending_requests(&self) -> Vec<ViewRequest> {
        let pending = self.pending_requests.read().await;
        pending.clone()
    }

    /// Get list of viewable images (received images with permissions)
    pub async fn get_viewable_images(&self) -> Vec<ViewableImageInfo> {
        let received = self.received_images.read().await;
        let mut viewable = Vec::new();

        for (image_id, path) in received.iter() {
            // Try to load and extract metadata
            if let Ok(img_bytes) = fs::read(path) {
                if let Ok(img) = image::load_from_memory(&img_bytes) {
                    let rgba = img.to_rgba8();
                    if let Ok(metadata) = extract_metadata(&rgba) {
                        if metadata.can_view(&self.username) {
                            let remaining = metadata.get_remaining_views(&self.username);
                            viewable.push(ViewableImageInfo {
                                image_id: image_id.clone(),
                                owner: metadata.owner.clone(),
                                remaining_views: remaining,
                                path: path.display().to_string(),
                            });
                        }
                    }
                }
            }
        }

        viewable
    }

    /// View an image and decrement the view count
    /// Returns (path, is_access_denied)
    pub async fn view_image_with_tracking(&self, image_id: &str) -> Result<(String, bool), Box<dyn std::error::Error>> {
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

                    return Ok((path_clone.display().to_string(), false));
                }
            }

            // Views exhausted - create and return access denied image
            let denied_img = create_access_denied_image();
            let denied_path = format!("temp_access_denied_{}.png", image_id);
            denied_img.save(&denied_path)?;
            return Ok((denied_path, true));
        }

        Err("Image not found".into())
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
            P2PRequest::RequestAccess {
                requester,
                requester_p2p_address,
                image_id,
                requested_views,
            } => {
                // Add to pending requests queue
                let mut pending = self.pending_requests.write().await;
                pending.push(ViewRequest {
                    requester: requester.clone(),
                    image_id: image_id.clone(),
                    requested_views,
                    requester_p2p_address,
                });
                drop(pending);

                println!("📥 New access request from {} for image '{}' ({} views)",
                    requester, image_id, requested_views);

                P2PResponse::RequestReceived {
                    message: format!(
                        "Your request for image '{}' has been sent to the owner",
                        image_id
                    ),
                }
            }
            P2PRequest::ApproveRequest {
                requester: _,
                image_id,
                granted_views,
                owner,
                owner_p2p_address,
            } => {
                // This approval is being received by the requester
                println!("✓ Access approved for image '{}' ({} views)", image_id, granted_views);
                println!("🔄 Auto-downloading approved image '{}'...", image_id);

                // Automatically download the image now that we have permission
                // Note: This download happens inline to ensure it completes
                match self.request_image_from_peer(&owner, &image_id, &owner_p2p_address).await {
                    Ok(_) => println!("✓ Image '{}' downloaded successfully and ready to view", image_id),
                    Err(e) => eprintln!("✗ Failed to download image '{}': {}", image_id, e),
                }

                P2PResponse::ApprovalSent {
                    message: format!(
                        "You have been granted {} views for image '{}'",
                        granted_views, image_id
                    ),
                }
            }
            P2PRequest::RequestThumbnail { requester: _, image_id } => {
                // Return a low-resolution thumbnail
                let owned = self.owned_images.read().await;
                if let Some(path) = owned.get(&image_id) {
                    let path_clone = path.clone();
                    drop(owned);

                    // Load and resize image to create thumbnail
                    match image::open(&path_clone) {
                        Ok(img) => {
                            // Create 100x100 thumbnail
                            let thumbnail = img.thumbnail(100, 100);
                            let mut buffer: Vec<u8> = Vec::new();
                            let mut cursor = std::io::Cursor::new(&mut buffer);

                            match thumbnail.write_to(&mut cursor, image::ImageFormat::Png) {
                                Ok(_) => P2PResponse::ThumbnailData {
                                    image_id,
                                    data: buffer,
                                },
                                Err(_) => P2PResponse::Error {
                                    message: "Failed to encode thumbnail".to_string(),
                                },
                            }
                        }
                        Err(_) => P2PResponse::Error {
                            message: "Failed to load image".to_string(),
                        },
                    }
                } else {
                    P2PResponse::Error {
                        message: "Image not found".to_string(),
                    }
                }
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
        eprintln!("  Auto-detect IP:   cargo run --bin client alice 9001");
        eprintln!("  Manual IP:        cargo run --bin client alice 9001 192.168.1.100");
        eprintln!();
        eprintln!("Note: IP address is optional. If not provided, it will be auto-detected");
        eprintln!("      by connecting to the discovery servers in config.toml");
        std::process::exit(1);
    }

    let username = args[1].clone();
    let p2p_port: u16 = args[2].parse().expect("Invalid port");

    let config = Config::load("config.toml").expect("Failed to load config");
    let server_addresses: Vec<String> = config.servers.values().cloned().collect();

    // Determine P2P address: auto-detect or use manual override
    let p2p_register_address = if args.len() > 3 {
        // Manual override: user provided IP address
        let p2p_ip = args[3].clone();
        println!("Using manually specified IP: {}", p2p_ip);
        format!("{}:{}", p2p_ip, p2p_port)
    } else {
        // Auto-detect: find our network IP by connecting to discovery servers
        println!("Auto-detecting network IP address...");

        match detect_local_ip(&server_addresses).await {
            Ok(detected_ip) => {
                let address = format_p2p_address(detected_ip, p2p_port);
                println!("✓ Detected P2P address: {}", address);
                address
            }
            Err(e) => {
                eprintln!("⚠ Auto-detection failed: {}", e);
                eprintln!("Falling back to localhost (127.0.0.1)");
                eprintln!("Note: P2P connections will only work locally.");
                eprintln!("For distributed deployment, provide IP manually:");
                eprintln!("  cargo run --bin client {} {} <your_ip>", username, p2p_port);
                format!("127.0.0.1:{}", p2p_port)
            }
        }
    };

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
