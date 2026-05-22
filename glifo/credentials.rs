use aes_gcm::{
    Aes256Gcm, Key,
    aead::{KeyInit, OsRng, rand_core::RngCore},
};
use chacha20poly1305::ChaCha20Poly1305;
use anyhow::Result;
use commom::constants::SYSTEM_DATA_FOLDER_PATH;
use pkcs8::der::zeroize::Zeroizing;
use std::path::Path;
use tokio::{fs, task};

const AES_KEY_SIZE: usize = 32;
const CHACHA_KEY_SIZE: usize = 32;
const SERPENT_KEY_SIZE: usize = 32;
const TWOFISH_KEY_SIZE: usize = 32;
const KEYS_FILENAME: &str = "keys";

pub struct Credentials {
    pub aes: Zeroizing<[u8; AES_KEY_SIZE]>,
    pub chacha: Zeroizing<[u8; CHACHA_KEY_SIZE]>,
    pub serpent: Zeroizing<[u8; SERPENT_KEY_SIZE]>,
    pub twofish: Zeroizing<[u8; TWOFISH_KEY_SIZE]>,
}

#[derive(Default, compactly::v1::Encode, Debug)]
pub struct SCredentials {
    pub aes: [u8; AES_KEY_SIZE],
    pub chacha: [u8; CHACHA_KEY_SIZE],
    pub serpent: [u8; SERPENT_KEY_SIZE],
    pub twofish: [u8; TWOFISH_KEY_SIZE],
}

pub async fn handle_credentials() -> Result<Credentials> {
    let existent_credentials_handler = task::spawn(search_existent_keys());

    // If keys exist, return them immediately
    if let Some(credentials) = existent_credentials_handler.await?? {
        return Ok(credentials);
    };

    // Otherwise, generate a fresh batch of keys for all 4 ciphers
    let credentials = Credentials {
        aes: Zeroizing::new(generate_aes_gcm_key()),
        chacha: Zeroizing::new(generate_chacha_key()),
        serpent: Zeroizing::new(generate_serpent_key()),
        twofish: Zeroizing::new(generate_twofish_key()),
    };

    // Save the completely new set of keys to the file system
    save_credentials_to_fs(&credentials).await?;
    Ok(credentials)
}

pub fn generate_aes_gcm_key() -> [u8; AES_KEY_SIZE] {
    let key: Key<Aes256Gcm> = Aes256Gcm::generate_key(OsRng);
    key.into()
}

fn generate_chacha_key() -> [u8; CHACHA_KEY_SIZE] {
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    key.into()
}

fn generate_twofish_key() -> [u8; TWOFISH_KEY_SIZE] {
    let mut key = [0u8; TWOFISH_KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

fn generate_serpent_key() -> [u8; SERPENT_KEY_SIZE] {
    let mut key = [0u8; SERPENT_KEY_SIZE]; 
    OsRng.fill_bytes(&mut key);
    key
}

pub async fn search_existent_keys() -> Result<Option<Credentials>> {
    let folder = Path::new(SYSTEM_DATA_FOLDER_PATH);
    let file = folder.join(KEYS_FILENAME);

    // Verify folder existence
    if let Ok(metadata) = fs::metadata(&folder).await {
        if !metadata.is_dir() { return Ok(None); }
    } else {
        return Ok(None);
    }

    // Verify file existence
    if let Ok(metadata) = fs::metadata(&file).await {
        if !metadata.is_file() { return Ok(None); }
    } else {
        return Ok(None);
    }

    let encoded_credentials = fs::read(file).await?;
    if let Some(credentials) = compactly::v1::decode::<SCredentials>(&encoded_credentials) {
        return Ok(Some(Credentials {
            aes: Zeroizing::new(credentials.aes),
            chacha: Zeroizing::new(credentials.chacha),
            serpent: Zeroizing::new(credentials.serpent),
            twofish: Zeroizing::new(credentials.twofish),
        }));
    }
    Ok(None)
}

pub async fn save_credentials_to_fs(credentials: &Credentials) -> Result<()> {
    let encoded_credentials = compactly::v1::encode(&SCredentials {
        aes: *credentials.aes,
        chacha: *credentials.chacha,
        serpent: *credentials.serpent,
        twofish: *credentials.twofish,
    });
    let file_path = Path::new(SYSTEM_DATA_FOLDER_PATH).join(KEYS_FILENAME);
    fs::write(file_path, encoded_credentials).await?;
    Ok(())
}