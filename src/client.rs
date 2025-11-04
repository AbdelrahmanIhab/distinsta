mod protocol;

use protocol::{ClientRequest, ServerResponse};
use std::env;
use std::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

struct Client {
    username: String,
    server_addresses: Vec<String>,
}

impl Client {
    fn new(username: String, server_addresses: Vec<String>) -> Self {
        Client {
            username,
            server_addresses,
        }
    }

    /// Broadcast request to all servers and wait for first successful response
    async fn broadcast_request(&self, request: ClientRequest) -> Result<ServerResponse, Box<dyn std::error::Error>> {
        println!("Broadcasting request to {} servers...", self.server_addresses.len());

        let request_json = serde_json::to_string(&request)?;

        // Send to all servers concurrently
        let mut tasks = vec![];

        for (idx, address) in self.server_addresses.iter().enumerate() {
            let addr = address.clone();
            let req = request_json.clone();

            let task = tokio::spawn(async move {
                println!("  Sending to server {} at {}", idx + 1, addr);

                match TcpStream::connect(&addr).await {
                    Ok(mut stream) => {
                        if stream.write_all(req.as_bytes()).await.is_err() {
                            return Err("Write failed".to_string());
                        }
                        if stream.write_all(b"\n").await.is_err() {
                            return Err("Write newline failed".to_string());
                        }

                        let mut reader = BufReader::new(&mut stream);
                        let mut response_line = String::new();

                        match reader.read_line(&mut response_line).await {
                            Ok(_) => {
                                match serde_json::from_str::<ServerResponse>(&response_line) {
                                    Ok(response) => Ok((idx + 1, response)),
                                    Err(e) => Err(format!("Parse error: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Read error: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Connection failed: {}", e)),
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks and collect results
        let mut successful_responses = vec![];
        for task in tasks {
            if let Ok(result) = task.await {
                if let Ok((server_id, response)) = result {
                    // Only accept non-error responses (from assigned server)
                    match &response {
                        ServerResponse::EncryptedImageData { .. } => {
                            println!("  ✓ Server {} processed request", server_id);
                            successful_responses.push(response);
                        }
                        ServerResponse::Error { message } => {
                            println!("  - Server {} declined: {}", server_id, message);
                        }
                    }
                }
            }
        }

        if successful_responses.is_empty() {
            return Err("No server processed the request (all servers declined)".into());
        }

        // Return the first successful response (from assigned server)
        Ok(successful_responses.into_iter().next().unwrap())
    }

    async fn upload_image(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Uploading Image ===");
        println!("File: {}", filepath);
        println!("User: {}", self.username);

        // Read image file
        let image_data = fs::read(filepath)?;
        let filename = std::path::Path::new(filepath)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        println!("Image size: {} bytes", image_data.len());

        let request = ClientRequest::UploadImage {
            username: self.username.clone(),
            image_data,
            filename: filename.clone(),
        };

        match self.broadcast_request(request).await? {
            ServerResponse::EncryptedImageData { data } => {
                // Save encrypted image to images directory with timestamp
                fs::create_dir_all("images")?;

                // Generate unique filename using timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let file_stem = std::path::Path::new(&filename)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let extension = std::path::Path::new(&filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png");

                let encrypted_path = format!("images/encrypted_{}_{}.{}",
                    file_stem, timestamp, extension);

                fs::write(&encrypted_path, data)?;
                println!("\n✓ Success!");
                println!("Encrypted image saved to: {}", encrypted_path);
            }
            ServerResponse::Error { message } => {
                eprintln!("\n✗ Error: {}", message);
            }
            _ => {
                eprintln!("\n✗ Unexpected response from server");
            }
        }

        Ok(())
    }

    async fn run_repl(&self) {
        println!("\n=== Distributed Image Storage Client (REPL) ===");
        println!("User: {}", self.username);
        println!("Multicast mode: Broadcasting to all servers");
        println!("Type 'help' for commands, 'quit' to exit");
        println!("================================================\n");

        loop {
            print!("{}> ", self.username);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }

                    match input {
                        "quit" | "exit" | "q" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" | "h" => {
                            println!("\nAvailable commands:");
                            println!("  upload <image_path>  - Upload and encrypt an image");
                            println!("  help                 - Show this help message");
                            println!("  quit                 - Exit the client\n");
                        }
                        _ if input.starts_with("upload ") => {
                            let parts: Vec<&str> = input.splitn(2, ' ').collect();
                            if parts.len() == 2 {
                                let image_path = parts[1].trim();
                                if let Err(e) = self.upload_image(image_path).await {
                                    eprintln!("Upload failed: {}\n", e);
                                }
                            } else {
                                eprintln!("Usage: upload <image_path>\n");
                            }
                        }
                        _ => {
                            eprintln!("Unknown command: '{}'. Type 'help' for available commands.\n", input);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <username>", args[0]);
        eprintln!("Example: {} alice", args[0]);
        eprintln!("\nNote: Client broadcasts to all servers (8001, 8002, 8003)");
        std::process::exit(1);
    }

    let username = args[1].clone();

    // Default server addresses - client multicasts to all
    let server_addresses = vec![
        "127.0.0.1:8001".to_string(),
        "127.0.0.1:8002".to_string(),
        "127.0.0.1:8003".to_string(),
    ];

    let client = Client::new(username, server_addresses);
    client.run_repl().await;
}
