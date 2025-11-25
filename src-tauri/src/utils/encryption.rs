use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use std::error::Error;

const NONCE_SIZE: usize = 12;

/// Generate a key from password and salt using SHA256
pub fn derive_key(password: &str, salt: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(data: &str, password: &str, salt: &str) -> Result<String, Box<dyn Error>> {
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key.into());

    // Generate a random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Combine nonce + ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    // Encode to base64
    Ok(general_purpose::STANDARD.encode(result))
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(encrypted_data: &str, password: &str, salt: &str) -> Result<String, Box<dyn Error>> {
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key.into());

    // Decode from base64
    let data = general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if data.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".into());
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode failed: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "my_secure_password";
        let salt = "random_salt_12345";
        let data = "Hello, World!";

        let encrypted = encrypt(data, password, salt).unwrap();
        let decrypted = decrypt(&encrypted, password, salt).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let salt = "random_salt";
        let data = "Secret data";

        let encrypted = encrypt(data, password, salt).unwrap();
        let result = decrypt(&encrypted, wrong_password, salt);

        assert!(result.is_err());
    }
}
