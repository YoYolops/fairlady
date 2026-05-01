use std::path::PathBuf;

use bytes::Bytes;
use tokio::{task};
use std::io::Cursor;
use tar::Archive;
use aes_gcm::{
    aead::{KeyInit, OsRng, Aead, AeadCore},
    Aes256Gcm, Nonce
};
use anyhow::{Result, bail};
use crate::credentials::{Credentials};
use commom::{self, constants::SYSTEM_FOREIGN_DATA_PATH};

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

pub async fn encrypt_user_data(credentials: &Credentials) -> Result<Vec<u8>> {
    // Encrypts all data inside ./data folder
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;
    let aes_session_key = credentials.aes.as_ref();
    let cipher = Aes256Gcm::new_from_slice(aes_session_key)
        .expect("Failed to build key from bytes");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_data = cipher.encrypt(&nonce, tar_data.as_ref())
        .expect("Failed to encrypt data");
    let mut payload = nonce.to_vec(); // prepend with nonce
    payload.extend_from_slice(&encrypted_data);
    Ok(payload)
}

pub async fn decrypt_foreign_data(credentials: &Credentials, encrypted_payload: &[u8]) -> Result<Vec<u8>> {
    if encrypted_payload.len() < 28 {
        bail!("Payload is too short to contain a valid nonce");
    }
    let (nonce_slice, ciphertext) = encrypted_payload.split_at(12); // extract nonce
    let nonce_array: [u8; 12] = nonce_slice.try_into().expect("Nonce slice is not exactly 12 bytes");
    let nonce = Nonce::from(nonce_array);
    let aes_session_key = credentials.aes.as_ref();
    let cipher = Aes256Gcm::new_from_slice(aes_session_key)
        .expect("Failed to build key from bytes");
    match cipher.decrypt(&nonce, ciphertext) {
        Ok(tar_data) => Ok(tar_data),
        Err(e) => {
            println!("{}", e);
            bail!("failed while decrypting")
        },
    }
}

pub async fn store_foreign_data(tar_data: Vec<u8>) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let cursor = Cursor::new(tar_data);
        let mut archive = Archive::new(cursor);
        archive.unpack(SYSTEM_FOREIGN_DATA_PATH)?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;
    println!("Successfully extracted foreign data to: {}", SYSTEM_FOREIGN_DATA_PATH);
    Ok(())
}