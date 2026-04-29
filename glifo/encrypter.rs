use std::path::PathBuf;

use tokio::task;
use aes_gcm::{
    aead::{KeyInit, OsRng, Aead, AeadCore},
    Aes256Gcm
};
use anyhow::Result;
use crate::credentials::{Credentials};
use commom;

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
    let hex_string = &tar_data
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(" ");
    println!("~ RAW DATA ~");
    println!("{}", hex_string);
    println!("~ END ~");
    let aes_session_key = credentials.aes.as_ref();
    let cipher = Aes256Gcm::new_from_slice(aes_session_key)
        .expect("Failed to build key from bytes");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_data = cipher.encrypt(&nonce, tar_data.as_ref())
        .expect("Failed to encrypt data");
    Ok(encrypted_data)
}