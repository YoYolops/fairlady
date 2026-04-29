use aes_gcm::{
    aead::{KeyInit, OsRng},
    Aes256Gcm, Key
};
use std::path::Path;
use pkcs8::der::zeroize::Zeroizing;
use anyhow::{Result, bail};
use commom::constants::SYSTEM_DATA_FOLDER_PATH;
use tokio::fs;

const AES_KEY_SIZE: usize = 32;
const KEYS_FILENAME: &str = "keys";

pub struct Credentials {
    pub aes: Zeroizing<[u8; AES_KEY_SIZE]>,
}

#[derive(Default, compactly::v1::Encode)]
pub struct SCredentials {
    pub aes: [u8; AES_KEY_SIZE],
}

// Creates key credentials for every algorithm the system requires.
// future work: store generated credentials and checkup if already existent on startup
pub async fn handle_credentials() -> Result<Credentials> {
    Ok(
        Credentials {
            aes: Zeroizing::new(generate_aes_gcm_key())
        }
    )
}

pub fn generate_aes_gcm_key() -> [u8; AES_KEY_SIZE] {
    let key: Key<Aes256Gcm> = Aes256Gcm::generate_key(OsRng);
    let converted_key: [u8; AES_KEY_SIZE] = key.into();
    converted_key
}

pub async fn search_existent_keys() -> Result<Option<Credentials>> {
    let folder = Path::new(SYSTEM_DATA_FOLDER_PATH);
    let file = folder.join(KEYS_FILENAME);
    {
        let metadata = fs::metadata(&folder).await?;
        if !metadata.is_dir() {
            return  Ok(None)
        }
    }
    {
        let metadata = fs::metadata(&file).await?;
        if !metadata.is_file() {
            return  Ok(None)
        }
    }
    let encoded_credentials = fs::read(file).await?;
    if let Some(credentials) = compactly::v1::decode::<SCredentials>(&encoded_credentials) {
        return Ok(Some(Credentials {
            aes: Zeroizing::new(credentials.aes)
        }))
    }
    bail!("Could not decode keys. Corrupted file or unknown format");
}

pub async fn save_credentials_to_fs(credentials: Credentials) -> Result<()> {
    let encoded_credentials = compactly::v1::encode(&SCredentials {
        aes: *credentials.aes
    });
    fs::write(
        &format!("{}/{}", SYSTEM_DATA_FOLDER_PATH, KEYS_FILENAME),
        encoded_credentials
    ).await?;
    Ok(())
}