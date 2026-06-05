use crate::credentials::Credentials;
use aes::Aes256;
use anyhow::{Result, anyhow, bail};
use chacha20::ChaCha20;
use commom::{
    self,
    constants::SYSTEM_FOREIGN_DATA_PATH,
    database::{Operation, PerformancePoint},
};
use ctr::{
    Ctr128BE,
    cipher::{KeyIvInit, StreamCipher},
};
use rand_core::{OsRng, TryRngCore};
use serpent::Serpent;
use std::io::Cursor;
use std::path::PathBuf;
use tar::Archive;
use tokio::task;
use twofish::Twofish;

type Aes256Ctr = Ctr128BE<Aes256>;
type SerpentCtr = Ctr128BE<Serpent>;
type TwofishCtr = Ctr128BE<Twofish>;

pub struct CryptoResult {
    pub data: Vec<u8>,
    pub perf_point: Option<PerformancePoint>,
}

#[derive(Debug)]
pub enum CryptoAlgorithm {
    AES,
    ChaCha20,
    Serpent,
    Twofish,
}

impl std::fmt::Display for CryptoAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stringy = match self {
            CryptoAlgorithm::AES => "aes",
            CryptoAlgorithm::ChaCha20 => "chacha",
            CryptoAlgorithm::Serpent => "serpent",
            CryptoAlgorithm::Twofish => "twofish",
        };
        write!(f, "{}", stringy)
    }
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

pub async fn encrypt_user_data(
    credentials: &Credentials,
    strategy: &CryptoAlgorithm,
) -> Result<CryptoResult> {
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;
    let mut perf_point = PerformancePoint {
        strategy: strategy.to_string(),
        final_timestamp: None,
        init_timestamp: None,
        operation: Operation::Encryption,
        payload_size: tar_data.len() as i64,
    };
    match strategy {
        CryptoAlgorithm::AES => {
            let aes_session_key = credentials.aes.as_ref();
            Ok(CryptoResult {
                data: encrypt_aes(aes_session_key, tar_data, &mut perf_point)?,
                perf_point: Some(perf_point),
            })
        }
        CryptoAlgorithm::ChaCha20 => {
            let chacha_key = credentials.chacha.as_ref();
            Ok(CryptoResult {
                data: encrypt_chacha(chacha_key, tar_data, &mut perf_point)?,
                perf_point: Some(perf_point),
            })
        }
        CryptoAlgorithm::Twofish => {
            let twofish_key = credentials.twofish.as_ref();
            Ok(CryptoResult {
                data: encrypt_twofish(twofish_key, tar_data, &mut perf_point)?,
                perf_point: Some(perf_point),
            })
        }
        CryptoAlgorithm::Serpent => {
            let serpent_key = credentials.serpent.as_ref();
            Ok(CryptoResult {
                data: encrypt_serpent(serpent_key, tar_data, &mut perf_point)?,
                perf_point: Some(perf_point),
            })
        }
    }
}

pub fn encrypt_aes(
    key: &[u8],
    mut data: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher =
        Aes256Ctr::new_from_slices(key, &nonce).expect("Invalid key or nonce length for AES");

    perf_point.clock_in()?;
    // Encrypt in place
    cipher.apply_keystream(&mut data);
    perf_point.clock_out()?;

    data.splice(0..0, nonce);

    Ok(data)
}

pub fn encrypt_chacha(
    key: &[u8],
    mut data: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 12];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher =
        ChaCha20::new_from_slices(key, &nonce).expect("Invalid key or nonce length for ChaCha20");

    perf_point.clock_in()?;
    // Encrypt in place
    cipher.apply_keystream(&mut data);
    perf_point.clock_out()?;

    data.splice(0..0, nonce);
    Ok(data)
}

fn encrypt_twofish(
    key: &[u8],
    mut data: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher =
        TwofishCtr::new_from_slices(key, &nonce).expect("Invalid key or nonce length for Twofish");

    perf_point.clock_in()?;
    // Encrypt in place
    cipher.apply_keystream(&mut data);
    perf_point.clock_out()?;

    data.splice(0..0, nonce);
    Ok(data)
}

fn encrypt_serpent(
    key: &[u8],
    mut data: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 16];
    let _ = OsRng.try_fill_bytes(&mut nonce);

    let mut cipher =
        SerpentCtr::new_from_slices(key, &nonce).expect("Invalid key or nonce length for Serpent");

    perf_point.clock_in()?;
    // Encrypt in place
    cipher.apply_keystream(&mut data);
    perf_point.clock_out()?;

    data.splice(0..0, nonce);
    Ok(data)
}

pub async fn decrypt_foreign_data(
    credentials: &Credentials,
    encrypted_payload: Vec<u8>,
    strategy: &CryptoAlgorithm,
) -> Result<CryptoResult> {
    // Initialize performance point for decryption tracking
    let mut perf_point = PerformancePoint {
        strategy: strategy.to_string(),
        final_timestamp: None,
        init_timestamp: None,
        operation: Operation::Decryption,
        payload_size: encrypted_payload.len() as i64,
    };

    let decrypted_data = match strategy {
        CryptoAlgorithm::AES => {
            let key = credentials.aes.as_ref();
            decrypt_aes(key, encrypted_payload, &mut perf_point)?
        }
        CryptoAlgorithm::ChaCha20 => {
            let key = credentials.chacha.as_ref();
            decrypt_chacha(key, encrypted_payload, &mut perf_point)?
        }
        CryptoAlgorithm::Twofish => {
            let key = credentials.twofish.as_ref();
            decrypt_twofish(key, encrypted_payload, &mut perf_point)?
        }
        CryptoAlgorithm::Serpent => {
            let key = credentials.serpent.as_ref();
            decrypt_serpent(key, encrypted_payload, &mut perf_point)?
        }
    };

    Ok(CryptoResult {
        data: decrypted_data,
        perf_point: Some(perf_point),
    })
}

fn decrypt_aes(
    key: &[u8],
    mut payload: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid AES nonce");
    }

    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&payload[..16]);

    let mut cipher = Aes256Ctr::new_from_slices(key, &nonce)
        .map_err(|_| anyhow!("Invalid AES key or nonce length"))?;

    perf_point.clock_in()?;
    // Decrypt directly into the ciphertext slice
    cipher.apply_keystream(&mut payload[16..]);
    perf_point.clock_out()?;

    // Drop the nonce, sliding the plaintext to the front
    payload.drain(..16);
    Ok(payload)
}

fn decrypt_chacha(
    key: &[u8],
    mut payload: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    if payload.len() < 12 {
        bail!("Payload is too short to contain a valid ChaCha nonce");
    }

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&payload[..12]);

    let mut cipher = ChaCha20::new_from_slices(key, &nonce)
        .map_err(|_| anyhow!("Invalid ChaCha key or nonce length"))?;

    perf_point.clock_in()?;
    cipher.apply_keystream(&mut payload[12..]);
    perf_point.clock_out()?;

    payload.drain(..12);
    Ok(payload)
}

fn decrypt_twofish(
    key: &[u8],
    mut payload: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid Twofish nonce");
    }

    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&payload[..16]);

    let mut cipher = TwofishCtr::new_from_slices(key, &nonce)
        .map_err(|_| anyhow!("Invalid Twofish key or nonce length"))?;

    perf_point.clock_in()?;
    cipher.apply_keystream(&mut payload[16..]);
    perf_point.clock_out()?;

    payload.drain(..16);
    Ok(payload)
}

fn decrypt_serpent(
    key: &[u8],
    mut payload: Vec<u8>,
    perf_point: &mut PerformancePoint,
) -> Result<Vec<u8>> {
    if payload.len() < 16 {
        bail!("Payload is too short to contain a valid Serpent nonce");
    }

    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&payload[..16]);

    let mut cipher = SerpentCtr::new_from_slices(key, &nonce)
        .map_err(|_| anyhow!("Invalid Serpent key or nonce length"))?;

    perf_point.clock_in()?;
    cipher.apply_keystream(&mut payload[16..]);
    perf_point.clock_out()?;

    payload.drain(..16);
    Ok(payload)
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
