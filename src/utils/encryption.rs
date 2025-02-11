use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine;
use keyring::Entry;
use rand::RngCore;

pub struct Enc;

impl Enc {
    pub fn encrypt(data: &str) -> Result<String, &'static str> {
        let key_bytes = Enc::get_encryption_key()?;
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

        let encrypted = base64::engine::general_purpose::STANDARD.encode(&encrypted_data);
        Ok(encrypted)
    }

    pub fn decrypt(secret: &str) -> Result<String, &'static str> {
        let key_bytes = Enc::get_encryption_key()?;
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

        let decrypted = String::from_utf8(decrypted).expect("Failed to convert decrypted data to string");
        Ok(decrypted)
    }

    fn get_encryption_key() -> Result<Vec<u8>, &'static str> {
        let entry = Entry::new("clai", "encryption_key").expect("Failed to create keyring entry");

        if let Ok(key) = entry.get_secret() {
            return Ok(key);
        }

        let mut key = vec![0u8; 32];
        rand::rng().fill_bytes(&mut key);
    
        match entry.set_secret(&key) {
            Ok(_) => Ok(key),
            Err(e) => match e {
                keyring::Error::PlatformFailure(_) => Err("No keyring service available. Please install a keyring service such as `gnome-keyring`."),
                keyring::Error::NoStorageAccess(_) => Err("No storage access. Please allow access to the keyring service."),
                _ => Err("Unknown encryption error"),
            }
        }
    }
}
