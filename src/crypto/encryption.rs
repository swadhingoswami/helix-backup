use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::Result;
use rand::RngCore;

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Valid AES-256-GCM key");
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            anyhow::bail!("Invalid encrypted data: too short");
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}

pub fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let mut key = [0u8; 32];
    key.copy_from_slice(hasher.finalize().as_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = Encryptor::generate_key();
        let encryptor = Encryptor::new(&key);

        let data = b"Hello, Helix encryption!";
        let encrypted = encryptor.encrypt(data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(data.to_vec(), decrypted);
        assert_ne!(data.to_vec(), encrypted);
    }

    #[test]
    fn test_different_keys() {
        let key1 = Encryptor::generate_key();
        let key2 = Encryptor::generate_key();

        let enc1 = Encryptor::new(&key1);
        let enc2 = Encryptor::new(&key2);

        let data = b"secret data";
        let encrypted = enc1.encrypt(data).unwrap();

        // Decrypting with wrong key should fail
        assert!(enc2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_key_derivation() {
        let salt = b"helix-salt-12345";
        let key1 = derive_key_from_password("password123", salt);
        let key2 = derive_key_from_password("password123", salt);
        let key3 = derive_key_from_password("different", salt);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
