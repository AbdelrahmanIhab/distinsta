use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub servers: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_server_address(&self, node_id: u32) -> Option<String> {
        let key = format!("node{}", node_id);
        self.servers.get(&key).cloned()
    }

    pub fn get_all_server_addresses(&self) -> Vec<String> {
        let mut addresses = vec![];
        for i in 1..=3 {
            if let Some(addr) = self.get_server_address(i) {
                addresses.push(addr);
            }
        }
        addresses
    }
}
