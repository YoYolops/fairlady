use std::path::PathBuf;

use crate::credentials::Credentials;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
};
use anyhow::{Result, bail};
use commom::{self, constants::SYSTEM_FOREIGN_DATA_PATH};
use std::io::Cursor;
use tar::Archive;
use tokio::task;
use chacha20poly1305::ChaCha20Poly1305;
use twofish::Twofish;
use serpent::Serpent;
use ctr::{Ctr128BE, cipher::{KeyIvInit, StreamCipher}};

type SerpentCtr = Ctr128BE<Serpent>;
type TwofishCtr = Ctr128BE<Twofish>;
pub enum CryptoAlgorithm {
    AES,
    ChaCha20,
    Serpent,
    Twofish
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
    })
    .await??;
    Ok(tar_result)
}

pub async fn encrypt_user_data(credentials: &Credentials, strategy: &CryptoAlgorithm) -> Result<Vec<u8>> {
    // Encrypts all data inside ./data folder
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;
    match strategy {
        CryptoAlgorithm::AES => {
            let aes_session_key = credentials.aes.as_ref();
            Ok(encrypt_aes(aes_session_key, tar_data))
        },
        CryptoAlgorithm::Twofish => {
            let twofish_key = credentials.twofish.as_ref();
            Ok(encrypt_twofish(twofish_key, tar_data))
        },
        CryptoAlgorithm::Serpent => {
            let serpent_key = credentials.serpent.as_ref();
            Ok(encrypt_serpent(serpent_key, tar_data))
        }
        _ => todo!("Not implemented other encryption strategy yet")
    }
}

fn encrypt_serpent(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    // Generate a 128-bit (16-byte) nonce/IV
    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);

    // Initialize the cipher
    let mut cipher = SerpentCtr::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length");
        
    // Encrypt the data in-place
    cipher.apply_keystream(&mut data);

    let mut payload = nonce.to_vec(); // Prepend with nonce
    payload.extend_from_slice(&data); // data is now ciphertext
    payload
}

fn encrypt_twofish(key: &[u8], mut data: Vec<u8>) -> Vec<u8> {
    // Generate a 128-bit (16-byte) nonce/IV
    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);

    // Initialize the cipher
    let mut cipher = TwofishCtr::new_from_slices(key, &nonce)
        .expect("Invalid key or nonce length");
        
    // Encrypt the data in-place
    cipher.apply_keystream(&mut data);

    let mut payload = nonce.to_vec(); // Prepend with nonce
    payload.extend_from_slice(&data); // data is now ciphertext
    payload
}

fn encrypt_chacha(key: &[u8], data: Vec<u8>) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .expect("Failed to build ChaCha key from bytes");
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); 
    let encrypted_data = cipher
        .encrypt(&nonce, data.as_ref())
        .expect("Failed to encrypt data");
        
    let mut payload = nonce.to_vec(); // Prepend with nonce
    payload.extend_from_slice(&encrypted_data);
    payload
}

fn encrypt_aes(key: &[u8], data: Vec<u8>) -> Vec<u8> {
    let cipher =
        Aes256Gcm::new_from_slice(key).expect("Failed to build aes key from bytes");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_data = cipher
        .encrypt(&nonce, data.as_ref())
        .expect("Failed to encrypt data");
    let mut payload = nonce.to_vec(); // prepend with nonce
    payload.extend_from_slice(&encrypted_data);
    payload
}

pub async fn decrypt_foreign_data(
    credentials: &Credentials,
    encrypted_payload: &[u8],
) -> Result<Vec<u8>> {
    if encrypted_payload.len() < 28 {
        bail!("Payload is too short to contain a valid nonce");
    }
    let (nonce_slice, ciphertext) = encrypted_payload.split_at(12); // extract nonce
    let nonce_array: [u8; 12] = nonce_slice
        .try_into()
        .expect("Nonce slice is not exactly 12 bytes");
    let nonce = Nonce::from(nonce_array);
    let aes_session_key = credentials.aes.as_ref();
    let cipher =
        Aes256Gcm::new_from_slice(aes_session_key).expect("Failed to build key from bytes");
    match cipher.decrypt(&nonce, ciphertext) {
        Ok(tar_data) => Ok(tar_data),
        Err(e) => {
            println!("{}", e);
            bail!("failed while decrypting")
        }
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
    println!(
        "Successfully extracted foreign data to: {}",
        SYSTEM_FOREIGN_DATA_PATH
    );
    Ok(())
}
