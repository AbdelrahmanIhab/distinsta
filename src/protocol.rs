use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    /// Upload an image - returns encrypted image data
    UploadImage {
        username: String,
        image_data: Vec<u8>,
        filename: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    /// Returns the encrypted image data
    EncryptedImageData { data: Vec<u8> },
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
