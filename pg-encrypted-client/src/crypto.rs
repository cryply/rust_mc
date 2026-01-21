//! Encryption Service - Equivalent to the "Enhanced Client Driver" 
//!
//! This module handles all cryptographic operations client-side, ensuring that
//! plaintext data never reaches the database server.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use thiserror::Error;

/// Encryption errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
}

/// Column Encryption Key (CEK) - encrypts actual data
/// This mirrors the "Database Encryption Key" from slide 1
pub struct ColumnEncryptionKey {
    key: [u8; 32],
}

impl ColumnEncryptionKey {
    /// Create a new random CEK
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// Derive CEK from a master key and column name (deterministic)
    pub fn derive(master_key: &MasterKey, column_name: &str) -> Result<Self, CryptoError> {
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                column_name.as_bytes(),
                &master_key.key,
                &mut key,
            )
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        Ok(Self { key })
    }

    /// Export as base64 (for storing encrypted in database)
    pub fn to_base64(&self) -> String {
        BASE64.encode(self.key)
    }

    /// Import from base64
    pub fn from_base64(encoded: &str) -> Result<Self, CryptoError> {
        let bytes = BASE64
            .decode(encoded)
            .map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(CryptoError::InvalidFormat("Key must be 32 bytes".into()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }
}

/// Column Master Key (CMK) - encrypts Column Encryption Keys

pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// Derive master key from password (like DPAPI deriving from Windows credentials)
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self, CryptoError> {
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        Ok(Self { key })
    }

    /// Create from raw bytes (e.g., loaded from secure storage)
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self { key: *bytes }
    }

    /// Encrypt a Column Encryption Key
    pub fn encrypt_cek(&self, cek: &ColumnEncryptionKey) -> Result<String, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, cek.key.as_slice())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// Decrypt a Column Encryption Key
    pub fn decrypt_cek(&self, encrypted: &str) -> Result<ColumnEncryptionKey, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;

        if combined.len() < 12 {
            return Err(CryptoError::InvalidFormat("Ciphertext too short".into()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        if plaintext.len() != 32 {
            return Err(CryptoError::InvalidFormat("Decrypted key wrong size".into()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&plaintext);
        Ok(ColumnEncryptionKey { key })
    }
}

/// The Enhanced Client Driver - handles all encryption/decryption
pub struct EncryptedClientDriver {
    cipher: Aes256Gcm,
}

impl EncryptedClientDriver {
    /// Create driver with a Column Encryption Key
    pub fn new(cek: &ColumnEncryptionKey) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&cek.key)
            .expect("Invalid key length - should never happen with 32-byte key");
        Self { cipher }
    }

    /// Encrypt plaintext data before sending to database
    /// Returns base64-encoded ciphertext (nonce prepended)
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// Decrypt ciphertext received from database
    /// Expects base64-encoded data with nonce prepended
    pub fn decrypt(&self, encrypted: &str) -> Result<String, CryptoError> {
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;

        if combined.len() < 12 {
            return Err(CryptoError::InvalidFormat(
                "Ciphertext too short - must include 12-byte nonce".into(),
            ));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid UTF-8: {e}")))
    }

    /// Encrypt optional field
    pub fn encrypt_optional(&self, plaintext: Option<&str>) -> Result<Option<String>, CryptoError> {
        plaintext.map(|p| self.encrypt(p)).transpose()
    }

    /// Decrypt optional field
    pub fn decrypt_optional(&self, encrypted: Option<&str>) -> Result<Option<String>, CryptoError> {
        encrypted.map(|e| self.decrypt(e)).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let cek = ColumnEncryptionKey::generate();
        let driver = EncryptedClientDriver::new(&cek);

        let plaintext = "Sensitive data: SSN 123-45-6789";
        let encrypted = driver.encrypt(plaintext).unwrap();
        let decrypted = driver.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_ne!(plaintext, encrypted); // Ensure it's actually encrypted
    }

    #[test]
    fn test_master_key_cek_hierarchy() {
        // Simulate the key hierarchy from slide 1
        let master_key = MasterKey::from_password("super_secret_password", b"salt1234salt1234").unwrap();
        
        // Create and encrypt a CEK
        let cek = ColumnEncryptionKey::generate();
        let encrypted_cek = master_key.encrypt_cek(&cek).unwrap();
        
        // Decrypt the CEK
        let decrypted_cek = master_key.decrypt_cek(&encrypted_cek).unwrap();
        
        // Both CEKs should produce same encryption results
        let driver1 = EncryptedClientDriver::new(&cek);
        let driver2 = EncryptedClientDriver::new(&decrypted_cek);
        
        let plaintext = "Test data";
        let encrypted = driver1.encrypt(plaintext).unwrap();
        
        // Note: Can't compare encrypted values directly due to random nonces
        // But decryption should work with both drivers
        assert_eq!(driver2.decrypt(&encrypted).unwrap(), plaintext);
    }

    #[test]
    fn test_derived_cek_deterministic() {
        let master_key = MasterKey::from_password("password", b"saltsaltsaltsalt").unwrap();
        
        let cek1 = ColumnEncryptionKey::derive(&master_key, "users.email").unwrap();
        let cek2 = ColumnEncryptionKey::derive(&master_key, "users.email").unwrap();
        
        // Same derivation should produce same key
        assert_eq!(cek1.to_base64(), cek2.to_base64());
        
        // Different column should produce different key
        let cek3 = ColumnEncryptionKey::derive(&master_key, "users.ssn").unwrap();
        assert_ne!(cek1.to_base64(), cek3.to_base64());
    }
}
