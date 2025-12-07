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
    // Need 32 pixels for length (4 bytes * 8 bits) + 8 pixels per metadata byte
    let required_pixels = 32 + (metadata_bytes.len() * 8);
    let total_pixels = (image.width() * image.height() * 4) as usize; // 4 channels per pixel
    if required_pixels > total_pixels {
        return Err(format!("Image too small: need {} pixels, have {}", required_pixels, total_pixels));
    }

    let mut new_image = image.clone();
    let pixels = new_image.as_mut();

    // Embed metadata length in first 32 pixels (4 bytes, 8 bits each)
    let len = metadata_bytes.len() as u32;
    let len_bytes = len.to_le_bytes();
    for (byte_idx, &len_byte) in len_bytes.iter().enumerate() {
        for bit_idx in 0..8 {
            let pixel_pos = byte_idx * 8 + bit_idx;
            if pixel_pos < pixels.len() {
                let bit_value = (len_byte >> bit_idx) & 1;
                pixels[pixel_pos] = (pixels[pixel_pos] & 0xFE) | bit_value;
            }
        }
    }

    // Embed metadata starting after length (32 pixels)
    for (byte_idx, &byte) in metadata_bytes.iter().enumerate() {
        for bit_idx in 0..8 {
            let pixel_pos = 32 + byte_idx * 8 + bit_idx;
            if pixel_pos < pixels.len() {
                let bit_value = (byte >> bit_idx) & 1;
                pixels[pixel_pos] = (pixels[pixel_pos] & 0xFE) | bit_value;
            }
        }
    }

    Ok(new_image)
}

/// Extract metadata from an image using LSB steganography
pub fn extract_metadata(image: &RgbaImage) -> Result<ImageMetadata, String> {
    let pixels = image.as_raw();

    // Extract metadata length from first 32 pixels (4 bytes, 8 bits each)
    let mut len_bytes = [0u8; 4];
    for byte_idx in 0..4 {
        let mut byte = 0u8;
        for bit_idx in 0..8 {
            let pixel_pos = byte_idx * 8 + bit_idx;
            if pixel_pos >= pixels.len() {
                return Err("Image too small".to_string());
            }
            let bit_value = pixels[pixel_pos] & 1;
            byte |= bit_value << bit_idx;
        }
        len_bytes[byte_idx] = byte;
    }
    let metadata_len = u32::from_le_bytes(len_bytes) as usize;

    if metadata_len == 0 || metadata_len > 1_000_000 {
        return Err(format!("Invalid metadata length: {}", metadata_len));
    }

    // Extract metadata bytes starting after length (32 pixels)
    let mut metadata_bytes = Vec::new();
    for byte_idx in 0..metadata_len {
        let mut byte = 0u8;
        for bit_idx in 0..8 {
            let pixel_pos = 32 + byte_idx * 8 + bit_idx;
            if pixel_pos < pixels.len() {
                let bit_value = pixels[pixel_pos] & 1;
                byte |= bit_value << bit_idx;
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
