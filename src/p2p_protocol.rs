use serde::{Deserialize, Serialize};
use crate::discovery::ImageInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PRequest {
    /// Request an image from a peer
    RequestImage {
        requester: String,
        image_id: String,
    },
    /// Get list of images owned by a peer
    GetImageList {
        requester: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PResponse {
    /// Image data with embedded metadata
    ImageData {
        image_id: String,
        data: Vec<u8>,
    },
    /// List of available images
    ImageList {
        images: Vec<ImageInfo>,
    },
    /// Access denied
    AccessDenied {
        reason: String,
    },
    /// Error occurred
    Error {
        message: String,
    },
}
