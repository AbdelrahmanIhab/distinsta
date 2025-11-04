use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ServerLoad {
    pub server_id: u32,
    pub address: String,
    pub current_load: usize,
    pub available: bool,
}

pub struct LoadBalancer {
    pub servers: Arc<RwLock<HashMap<u32, ServerLoad>>>,
    pub next_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        LoadBalancer {
            servers: Arc::new(RwLock::new(HashMap::new())),
            next_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a server with the load balancer
    pub async fn register_server(&self, server_id: u32, address: String) {
        println!("LoadBalancer: Registered server {} at {}", server_id, address);
        let mut servers = self.servers.write().await;
        servers.insert(
            server_id,
            ServerLoad {
                server_id,
                address,
                current_load: 0,
                available: true,
            },
        );
    }

    /// Remove a server from the load balancer
    pub async fn unregister_server(&self, server_id: u32) {
        let mut servers = self.servers.write().await;
        servers.remove(&server_id);
        println!("LoadBalancer: Unregistered server {}", server_id);
    }

    /// Get the next available server using round-robin
    pub async fn get_next_server(&self) -> Option<(u32, String)> {
        let servers = self.servers.read().await;

        if servers.is_empty() {
            return None;
        }

        let available_servers: Vec<_> = servers
            .values()
            .filter(|s| s.available)
            .collect();

        if available_servers.is_empty() {
            return None;
        }

        let mut next_idx = self.next_index.write().await;
        let server = &available_servers[*next_idx % available_servers.len()];
        *next_idx = (*next_idx + 1) % available_servers.len();

        Some((server.server_id, server.address.clone()))
    }

    /// Get server with least load
    pub async fn get_least_loaded_server(&self) -> Option<(u32, String)> {
        let servers = self.servers.read().await;

        servers
            .values()
            .filter(|s| s.available)
            .min_by_key(|s| s.current_load)
            .map(|s| (s.server_id, s.address.clone()))
    }

    /// Update server load
    pub async fn update_server_load(&self, server_id: u32, load: usize) {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(&server_id) {
            server.current_load = load;
        }
    }

    /// Mark server as unavailable
    pub async fn mark_server_unavailable(&self, server_id: u32) {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(&server_id) {
            server.available = false;
            println!("LoadBalancer: Marked server {} as unavailable", server_id);
        }
    }

    /// Mark server as available
    pub async fn mark_server_available(&self, server_id: u32) {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(&server_id) {
            server.available = true;
            println!("LoadBalancer: Marked server {} as available", server_id);
        }
    }

    /// Get all available servers
    pub async fn get_available_servers(&self) -> Vec<(u32, String)> {
        let servers = self.servers.read().await;
        servers
            .values()
            .filter(|s| s.available)
            .map(|s| (s.server_id, s.address.clone()))
            .collect()
    }

    /// Get server count
    pub async fn get_server_count(&self) -> usize {
        let servers = self.servers.read().await;
        servers.values().filter(|s| s.available).count()
    }
}
