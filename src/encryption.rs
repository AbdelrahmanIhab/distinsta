use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use sha2::{Digest, Sha256};

type Aes128Ctr = Ctr128BE<Aes128>;

/// Simple AES encryption for image data
pub fn encrypt_data(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let mut encrypted = data.to_vec();
    let iv = [0u8; 16]; // Simple IV for demo purposes
    let mut cipher = Aes128Ctr::new(key.into(), &iv.into());
    cipher.apply_keystream(&mut encrypted);
    encrypted
}

/// Simple AES decryption for image data
pub fn decrypt_data(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let mut decrypted = data.to_vec();
    let iv = [0u8; 16];
    let mut cipher = Aes128Ctr::new(key.into(), &iv.into());
    cipher.apply_keystream(&mut decrypted);
    decrypted
}

/// Generate a simple key from username
pub fn generate_key_from_username(username: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 16];
    key.copy_from_slice(&result[0..16]);
    key
}
