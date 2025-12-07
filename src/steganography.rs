use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub owner: String,
    pub image_id: String,
    pub permissions: HashMap<String, u32>, // username -> remaining views
}

impl ImageMetadata {
    pub fn new(owner: String, image_id: String) -> Self {
        Self {
            owner,
            image_id,
            permissions: HashMap::new(),
        }
    }

    pub fn add_permission(&mut self, username: String, views: u32) {
        self.permissions.insert(username, views);
    }

    pub fn can_view(&self, username: &str) -> bool {
        if username == self.owner {
            return true;
        }
        self.permissions.get(username).map_or(false, |&v| v > 0)
    }

    pub fn decrement_view(&mut self, username: &str) -> bool {
        if username == self.owner {
            return true; // Owner can always view
        }
        if let Some(views) = self.permissions.get_mut(username) {
            if *views > 0 {
                *views -= 1;
                return true;
            }
        }
        false
    }

    pub fn get_remaining_views(&self, username: &str) -> u32 {
        if username == self.owner {
            return u32::MAX; // Owner has unlimited views
        }
        self.permissions.get(username).copied().unwrap_or(0)
    }
}

/// Embed metadata into an image using LSB steganography
pub fn embed_metadata(image: &RgbaImage, metadata: &ImageMetadata) -> Result<RgbaImage, String> {
    let metadata_json = serde_json::to_string(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    let metadata_bytes = metadata_json.as_bytes();

    // Check if image is large enough
    let required_pixels = (metadata_bytes.len() + 4) * 4; // +4 for length header
    if required_pixels > (image.width() * image.height()) as usize {
        return Err("Image too small for metadata".to_string());
    }

    let mut new_image = image.clone();
    let pixels = new_image.as_mut();

    // Embed metadata length in first 4 pixels (16 bytes for u32)
    let len = metadata_bytes.len() as u32;
    let len_bytes = len.to_le_bytes();
    for i in 0..4 {
        pixels[i] = (pixels[i] & 0xFE) | (len_bytes[i] & 1);
    }

    // Embed metadata starting from pixel 4
    for (i, &byte) in metadata_bytes.iter().enumerate() {
        let pixel_idx = (i + 4) * 4;
        if pixel_idx >= pixels.len() {
            break;
        }

        // Embed byte across 4 consecutive pixel channels using LSB
        for bit in 0..8 {
            let channel_idx = pixel_idx + (bit / 2);
            if channel_idx < pixels.len() {
                let bit_value = (byte >> bit) & 1;
                pixels[channel_idx] = (pixels[channel_idx] & 0xFE) | bit_value;
            }
        }
    }

    Ok(new_image)
}

/// Extract metadata from an image using LSB steganography
pub fn extract_metadata(image: &RgbaImage) -> Result<ImageMetadata, String> {
    let pixels = image.as_raw();

    // Extract metadata length from first 4 pixels
    let mut len_bytes = [0u8; 4];
    for i in 0..4 {
        if i >= pixels.len() {
            return Err("Image too small".to_string());
        }
        len_bytes[i] = pixels[i] & 1;
    }
    let metadata_len = u32::from_le_bytes(len_bytes) as usize;

    if metadata_len == 0 || metadata_len > 1_000_000 {
        return Err("Invalid metadata length".to_string());
    }

    // Extract metadata bytes
    let mut metadata_bytes = Vec::new();
    for i in 0..metadata_len {
        let pixel_idx = (i + 4) * 4;
        let mut byte = 0u8;

        for bit in 0..8 {
            let channel_idx = pixel_idx + (bit / 2);
            if channel_idx < pixels.len() {
                let bit_value = pixels[channel_idx] & 1;
                byte |= bit_value << bit;
            }
        }
        metadata_bytes.push(byte);
    }

    let metadata_json = String::from_utf8(metadata_bytes)
        .map_err(|e| format!("Invalid UTF-8: {}", e))?;

    serde_json::from_str(&metadata_json)
        .map_err(|e| format!("Failed to deserialize metadata: {}", e))
}

/// Create a default "access denied" image
pub fn create_access_denied_image() -> RgbaImage {
    let width = 400;
    let height = 300;
    let mut img = ImageBuffer::new(width, height);

    // Fill with red color
    for pixel in img.pixels_mut() {
        *pixel = Rgba([200, 50, 50, 255]);
    }

    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_extract_metadata() {
        let img = ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255]));

        let mut metadata = ImageMetadata::new("alice".to_string(), "img001".to_string());
        metadata.add_permission("bob".to_string(), 5);

        let embedded = embed_metadata(&img, &metadata).unwrap();
        let extracted = extract_metadata(&embedded).unwrap();

        assert_eq!(extracted.owner, "alice");
        assert_eq!(extracted.image_id, "img001");
        assert_eq!(extracted.get_remaining_views("bob"), 5);
    }

    #[test]
    fn test_view_decrement() {
        let mut metadata = ImageMetadata::new("alice".to_string(), "img001".to_string());
        metadata.add_permission("bob".to_string(), 3);

        assert_eq!(metadata.get_remaining_views("bob"), 3);
        assert!(metadata.decrement_view("bob"));
        assert_eq!(metadata.get_remaining_views("bob"), 2);
    }
}
