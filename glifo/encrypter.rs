use std::path::PathBuf;

use tokio::task;
use aes_gcm::{
    aead::{KeyInit, OsRng, Aead, AeadCore},
    Aes256Gcm, Key
};
use pkcs8::der::zeroize::Zeroizing;
use anyhow::Result;
use commom;

const AES_KEY_SIZE: usize = 32;

pub struct Credentials {
    aes: Zeroizing<[u8; AES_KEY_SIZE]>,
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

async fn folder_to_tar_bytes(folder_path: PathBuf) -> Result<Vec<u8>> {
    let tar_result = task::spawn_blocking(move || -> Result<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        {
            let mut archive = tar::Builder::new(&mut data);
            // Adds the entire directory tree to the binary stream: dangerously greedy
            archive.append_dir_all(".", folder_path)?;
            archive.finish()?;
        }
        Ok(data)
    }).await??;
    Ok(tar_result)
}

pub async fn encrypt_data(credentials: Credentials) -> Result<Vec<u8>> {
    // Encrypts all data inside ./data folder
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;

    let aes_session_key = credentials.aes.as_ref();
    let cipher = Aes256Gcm::new_from_slice(aes_session_key)
        .expect("Failed to build key from bytes");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_data = cipher.encrypt(&nonce, tar_data.as_ref())
        .expect("Failed to encrypt data");
    Ok(encrypted_data)
}