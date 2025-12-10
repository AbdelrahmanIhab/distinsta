use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub p2p_address: String, // IP:PORT for P2P connection
    pub registered_at: u64,
    pub last_heartbeat: u64,
}

impl UserInfo {
    pub fn new(username: String, p2p_address: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            username,
            p2p_address,
            registered_at: now,
            last_heartbeat: now,
        }
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn is_online(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Consider offline if no heartbeat for 60 seconds
        now - self.last_heartbeat < 60
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub image_id: String,
    pub filename: String,
    pub owner: String,
    pub shared_with: Vec<String>, // List of usernames with access
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserRegistry {
    pub users: HashMap<String, UserInfo>,
    pub images: HashMap<String, ImageInfo>, // image_id -> ImageInfo
}

impl UserRegistry {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            images: HashMap::new(),
        }
    }

    pub fn register_user(&mut self, username: String, p2p_address: String) -> bool {
        let user_info = UserInfo::new(username.clone(), p2p_address);
        self.users.insert(username, user_info);
        true
    }

    pub fn unregister_user(&mut self, username: &str) -> bool {
        self.users.remove(username).is_some()
    }

    pub fn update_heartbeat(&mut self, username: &str) -> bool {
        if let Some(user) = self.users.get_mut(username) {
            user.update_heartbeat();
            true
        } else {
            false
        }
    }

    pub fn get_online_peers(&self, requester: &str) -> Vec<UserInfo> {
        self.users
            .values()
            .filter(|u| u.username != requester && u.is_online())
            .cloned()
            .collect()
    }

    pub fn get_user(&self, username: &str) -> Option<&UserInfo> {
        self.users.get(username)
    }

    pub fn publish_image(&mut self, image_info: ImageInfo) {
        self.images.insert(image_info.image_id.clone(), image_info);
    }

    pub fn get_user_images(&self, owner: &str) -> Vec<ImageInfo> {
        self.images
            .values()
            .filter(|img| img.owner == owner)
            .cloned()
            .collect()
    }

    pub fn get_all_images(&self) -> Vec<ImageInfo> {
        self.images.values().cloned().collect()
    }

    pub fn get_image(&self, image_id: &str) -> Option<&ImageInfo> {
        self.images.get(image_id)
    }

    pub fn update_image_permissions(&mut self, image_id: &str, shared_with: Vec<String>) -> bool {
        if let Some(image) = self.images.get_mut(image_id) {
            image.shared_with = shared_with;
            true
        } else {
            false
        }
    }

    pub fn remove_image(&mut self, image_id: &str) -> bool {
        self.images.remove(image_id).is_some()
    }

    pub fn get_all_online_users(&self) -> Vec<UserInfo> {
        self.users
            .values()
            .filter(|u| u.is_online())
            .cloned()
            .collect()
    }
}
