use std::path::PathBuf;
use std::io::Cursor;
use tar::Archive;
use tokio::task;
use anyhow::{anyhow, bail, Result};

use crate::credentials::Credentials;
use commom::{self, constants::SYSTEM_FOREIGN_DATA_PATH};

// Raw cryptography imports
use aes::Aes256;
use chacha20::ChaCha20;
use twofish::Twofish;
use serpent::Serpent;
use ctr::{Ctr128BE, cipher::{KeyIvInit, StreamCipher}};
use rand_core::{OsRng, TryRngCore};

// Define type aliases for the CTR mode ciphers
type Aes256Ctr = Ctr128BE<Aes256>;
type SerpentCtr = Ctr128BE<Serpent>;
type TwofishCtr = Ctr128BE<Twofish>;

pub enum CryptoAlgorithm {
    AES,
    ChaCha20,
    Serpent,
    Twofish,
}

async fn folder_to_tar_bytes(folder_path: PathBuf) -> Result<Vec<u8>> {
    let tar_result = task::spawn_blocking(move || -> Result<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        {
            let mut archive = tar::Builder::new(&mut data);
            archive.append_dir_all(".", folder_path)?;
            archive.finish()?;
        }
        Ok(data)
    })
    .await??;
    Ok(tar_result)
}

pub async fn encrypt_user_data(credentials: &Credentials, strategy: &CryptoAlgorithm) -> Result<Vec<u8>> {
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;
    
    match strategy {
        CryptoAlgorithm::AES => {
            let aes_session_key = credentials.aes.as_ref();
            Ok(encrypt_aes(aes_session_key, tar_data))
        },
        CryptoAlgorithm::ChaCha20 => {
            let chacha_key = credentials.chacha.as_ref();
            Ok(encrypt_chacha(chacha_key, tar_data))
        },
        CryptoAlgorithm::Twofish => {
            let twofish_key = credentials.twofish.as_ref();
            Ok(encrypt_twofish(twofish_key, tar_data))
        },
        CryptoAlgorithm::Serpent => {
            let serpent_key = credentials.serpent.as_ref();
            Ok(encrypt_serpent(serpent_key, tar_data))
        }
    }
}

pub fn encrypt_aes(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher = Aes256Ctr::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length for AES");
        
    cipher.apply_keystream(&mut data);

    let mut payload = Vec::with_capacity(nonce.len() + data.len());
    payload.extend_from_slice(&nonce);
    payload.append(&mut data);
    payload
}

pub fn encrypt_chacha(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher = ChaCha20::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length for ChaCha20");
        
    cipher.apply_keystream(&mut data);

    let mut payload = Vec::with_capacity(nonce.len() + data.len());
    payload.extend_from_slice(&nonce);
    payload.append(&mut data);
    payload
}

fn encrypt_twofish(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher = TwofishCtr::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length for Twofish");
        
    cipher.apply_keystream(&mut data);

    let mut payload = Vec::with_capacity(nonce.len() + data.len());
    payload.extend_from_slice(&nonce);
    payload.append(&mut data);
    payload
}

fn encrypt_serpent(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher = SerpentCtr::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length for Serpent");
        
    cipher.apply_keystream(&mut data);

    let mut payload = Vec::with_capacity(nonce.len() + data.len());
    payload.extend_from_slice(&nonce);
    payload.append(&mut data);
    payload
}

pub async fn decrypt_foreign_data(
    credentials: &Credentials,
    encrypted_payload: &[u8],
    strategy: &CryptoAlgorithm
) -> Result<Vec<u8>> {
    match strategy {
        CryptoAlgorithm::AES => {
            let key = credentials.aes.as_ref();
            decrypt_aes(key, encrypted_payload)
        }
        CryptoAlgorithm::ChaCha20 => {
            let key = credentials.chacha.as_ref();
            decrypt_chacha(key, encrypted_payload)
        }
        CryptoAlgorithm::Twofish => {
            let key = credentials.twofish.as_ref();
            decrypt_twofish(key, encrypted_payload)
        }
        CryptoAlgorithm::Serpent => {
            let key = credentials.serpent.as_ref();
            decrypt_serpent(key, encrypted_payload)
        }
    }
}

fn decrypt_aes(key: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid AES nonce");
    }
    let (nonce_slice, ciphertext) = payload.split_at(16);
    
    let mut data = ciphertext.to_vec();
    let mut cipher = Aes256Ctr::new_from_slices(key, nonce_slice)
        .map_err(|_| anyhow!("Invalid AES key or nonce length"))?;
        
    cipher.apply_keystream(&mut data);
    Ok(data)
}

fn decrypt_chacha(key: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 12 {
        bail!("Payload is too short to contain a valid ChaCha nonce");
    }
    let (nonce_slice, ciphertext) = payload.split_at(12);
    
    let mut data = ciphertext.to_vec();
    let mut cipher = ChaCha20::new_from_slices(key, nonce_slice)
        .map_err(|_| anyhow!("Invalid ChaCha key or nonce length"))?;
        
    cipher.apply_keystream(&mut data);
    Ok(data)
}

fn decrypt_twofish(key: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid Twofish nonce");
    }
    let (nonce_slice, ciphertext) = payload.split_at(16);
    
    let mut data = ciphertext.to_vec();
    let mut cipher = TwofishCtr::new_from_slices(key, nonce_slice)
        .map_err(|_| anyhow!("Invalid Twofish key or nonce length"))?;
        
    cipher.apply_keystream(&mut data);
    Ok(data)
}

fn decrypt_serpent(key: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid Serpent nonce");
    }
    let (nonce_slice, ciphertext) = payload.split_at(16);
    
    let mut data = ciphertext.to_vec();
    let mut cipher = SerpentCtr::new_from_slices(key, nonce_slice)
        .map_err(|_| anyhow!("Invalid Serpent key or nonce length"))?;
        
    cipher.apply_keystream(&mut data);
    Ok(data)
}

pub async fn store_foreign_data(tar_data: Vec<u8>) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let cursor = Cursor::new(tar_data);
        let mut archive = Archive::new(cursor);
        archive.unpack(SYSTEM_FOREIGN_DATA_PATH)?;
        Ok::<(), anyhow::Error>(())
    })
    .await??;
    
    println!(
        "Successfully extracted foreign data to: {}",
        SYSTEM_FOREIGN_DATA_PATH
    );
    Ok(())
}