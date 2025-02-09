use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine;
use keyring::Entry;
use rand::RngCore;

pub struct Enc;

impl Enc {
    pub fn encrypt(data: &str) -> String {
        let key_bytes = Enc::get_encryption_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = vec![0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .expect("Failed to encrypt data");

        let mut encrypted_data = nonce_bytes.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);

        base64::engine::general_purpose::STANDARD.encode(&encrypted_data)
    }

    pub fn decrypt(secret: &str) -> String {
        let key_bytes = Enc::get_encryption_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let encrypted_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret.as_bytes())
            .expect("Failed to decode base64");

        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = cipher
            .decrypt(nonce, ciphertext)
            .expect("Failed to decrypt data");

        String::from_utf8(decrypted).expect("Failed to convert decrypted data to string")
    }

    fn get_encryption_key() -> Vec<u8> {
        let entry = Entry::new("clai", "encryption_key").expect("Failed to create keyring entry");

        if let Ok(key) = entry.get_secret() {
            return key;
        }

        let mut key = vec![0u8; 32];
        rand::rng().fill_bytes(&mut key);
        entry.set_secret(&key).expect("Failed to set secret");
        key
    }
}
