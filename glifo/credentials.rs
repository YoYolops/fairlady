use aes_gcm::{
    aead::{KeyInit, OsRng},
    Aes256Gcm, Key
};
use std::path::Path;
use pkcs8::der::zeroize::Zeroizing;
use anyhow::{Result};
use commom::constants::SYSTEM_DATA_FOLDER_PATH;
use tokio::{fs, task};

const AES_KEY_SIZE: usize = 32;
const KEYS_FILENAME: &str = "keys";

pub struct Credentials {
    pub aes: Zeroizing<[u8; AES_KEY_SIZE]>,
}

#[derive(Default, compactly::v1::Encode, Debug)]
pub struct SCredentials {
    // struct to serialize credentials. Serializing Zeroizing type with compactly is problematic
    pub aes: [u8; AES_KEY_SIZE],
}

pub async fn handle_credentials() -> Result<Credentials> {
    let existent_credentials_handler = task::spawn(search_existent_keys());
    let new_key = generate_aes_gcm_key();

    if let Some(credentials) = existent_credentials_handler.await?? {
        return Ok(credentials)
    };
    let credentials = Credentials {
        aes: Zeroizing::new(new_key)
    };
    save_credentials_to_fs(&credentials).await?;
    Ok(credentials)
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
        if let Ok(metadata) = fs::metadata(&folder).await {
            if !metadata.is_dir() { return Ok(None) };
        } else { return Ok(None) };
    }
    {
        if let Ok(metadata) = fs::metadata(&file).await {
            if !metadata.is_file() { return Ok(None) };
        } else { return Ok(None) };

    }
    let encoded_credentials = fs::read(file).await?;
    if let Some(credentials) = compactly::v1::decode::<SCredentials>(&encoded_credentials) {
        println!("FOUND CREDENTIALS");
        println!("{:#?}", credentials);
        return Ok(Some(Credentials {
            aes: Zeroizing::new(credentials.aes)
        }))
    }
    Ok(None)
}

pub async fn save_credentials_to_fs(credentials: &Credentials) -> Result<()> {
    let encoded_credentials = compactly::v1::encode(&SCredentials {
        aes: *credentials.aes
    });
    fs::write(
        &format!("{}/{}", SYSTEM_DATA_FOLDER_PATH, KEYS_FILENAME),
        encoded_credentials
    ).await?;
    Ok(())
}