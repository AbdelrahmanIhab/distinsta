use serde::{Deserialize, Serialize};
use crate::discovery::{UserInfo, ImageInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    /// Upload an image - returns encrypted image data
    UploadImage {
        username: String,
        image_data: Vec<u8>,
        filename: String,
    },
    /// Register with discovery service
    Register {
        username: String,
        p2p_address: String,
    },
    /// Unregister from discovery service
    Unregister {
        username: String,
    },
    /// Get list of online peers
    GetPeers {
        username: String,
    },
    /// Publish image to discovery service
    PublishImage {
        image_info: ImageInfo,
    },
    /// Update image permissions
    UpdatePermissions {
        image_id: String,
        shared_with: Vec<String>,
    },
    /// Heartbeat to stay online
    Heartbeat {
        username: String,
    },
    /// Get images owned by a user
    GetUserImages {
        owner: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    /// Returns the encrypted image data
    EncryptedImageData { data: Vec<u8> },
    /// Registration success
    Registered { success: bool, message: String },
    /// List of online peers
    PeerList { peers: Vec<UserInfo> },
    /// Image published successfully
    ImagePublished { image_id: String },
    /// Permissions updated
    PermissionsUpdated { success: bool },
    /// Heartbeat acknowledged
    HeartbeatAck,
    /// List of images
    ImageList { images: Vec<ImageInfo> },
    /// Generic success
    Success { message: String },
    /// Error
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalMessage {
    /// Request from leader to worker to process image
    ProcessImage {
        username: String,
        image_data: Vec<u8>,
        filename: String,
    },
    /// Response from worker to leader
    ProcessingComplete { success: bool, message: String },
    /// Retrieve image from worker
    RetrieveImage { username: String, filename: String },
    /// Image retrieval response
    ImageData { data: Vec<u8> },
    /// Health check
    Ping,
    /// Health check response
    Pong,
}
