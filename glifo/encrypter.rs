use std::path::PathBuf;

use tokio::{
    self,
    task::{self, JoinHandle},
};
use aes_gcm::{
    aead::{KeyInit, OsRng, Aead, AeadCore},
    Aes256Gcm, Key, Nonce
};
use rsa::{
    Oaep,
    RsaPrivateKey,
    RsaPublicKey,
    pkcs8::{
        EncodePrivateKey,
        EncodePublicKey,
        LineEnding
    }
};
use pkcs8::der::zeroize::Zeroizing;
use p256::{
    ecdsa::{
        SigningKey,
        VerifyingKey
    }
};
use rand::{rngs::ThreadRng, thread_rng};
use anyhow::Result;
use sha2::{Sha256, Digest};
use commom;

const AES_KEY_SIZE: usize = 32;
pub struct RsaKeyPair {
    public: RsaPublicKey,
    private: RsaPrivateKey,
    pub_pem: String,
    priv_pem: Zeroizing<String>
}

pub struct EcdsaKeyPair {
    public: VerifyingKey,
    private: SigningKey,
    pub_pem: String,
    priv_pem: Zeroizing<String>
}

pub struct Credentials {
    rsa: RsaKeyPair,
    ecdsa: EcdsaKeyPair,
}

// Creates key credentials for every algorithm the system requires.
// future work: store generated credentials and checkup if already existent on startup
pub async fn handle_credentials() -> Result<Credentials> {
    let rsa_task_handler: JoinHandle<Result<RsaKeyPair>> = task::spawn_blocking(|| {
        let mut rng: ThreadRng = thread_rng();
        generate_rsa_key_pair(&mut rng)
    });
    let ecds_task_handler: JoinHandle<Result<EcdsaKeyPair>> = task::spawn_blocking(|| {
        let mut rng: ThreadRng = thread_rng();
        generate_ecdsa_key_pair(&mut rng)
    });

    let (rsa_result, ecdsa_result) = tokio::join!(rsa_task_handler, ecds_task_handler);

    let rsa_credentials: RsaKeyPair = rsa_result??;
    let ecdsa_credentials: EcdsaKeyPair = ecdsa_result??;

    Ok(
        Credentials {
            rsa: rsa_credentials,
            ecdsa: ecdsa_credentials
        }
    )
}

pub fn generate_rsa_key_pair(rng: &mut ThreadRng) -> Result<RsaKeyPair> {
    let rsa_priv: RsaPrivateKey = RsaPrivateKey::new(rng, 2048)?;
    let rsa_pub: RsaPublicKey = RsaPublicKey::from(&rsa_priv);

    let rsa_priv_pem: Zeroizing<String> = rsa_priv.to_pkcs8_pem(LineEnding::LF)?;
    let rsa_pub_pem: String = rsa_pub.to_public_key_pem(LineEnding::LF)?;

    Ok(
        RsaKeyPair {
            public: rsa_pub,
            private: rsa_priv,
            pub_pem: rsa_pub_pem,
            priv_pem: rsa_priv_pem
        }
    )
}

pub fn generate_ecdsa_key_pair(rng: &mut ThreadRng) -> Result<EcdsaKeyPair> {
    let ecdsa_priv = SigningKey::random(rng);
    let ecdsa_pub = ecdsa_priv.verifying_key().clone();

    let ecdsa_priv_pem: Zeroizing<String> = ecdsa_priv.to_pkcs8_pem(LineEnding::LF)?;
    let ecdsa_pub_pem: String = ecdsa_pub.to_public_key_pem(LineEnding::LF)?;

    Ok(
        EcdsaKeyPair {
            public: ecdsa_pub,
            private: ecdsa_priv,
            pub_pem: ecdsa_pub_pem,
            priv_pem: ecdsa_priv_pem
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

pub async fn encrypt_data(credentials: Credentials) -> Result<()> {
    // Encrypts all data inside ./data folder
    let userdata_path = commom::info::get_userdata_path()?;
    let tar_data = folder_to_tar_bytes(userdata_path).await?;

    // Ensure message integrity
    let tar_data_hash = Sha256::digest(&tar_data);

    // Ensure confidentiality
    let aes_session_key = Zeroizing::new(generate_aes_gcm_key());
    let cipher = Aes256Gcm::new_from_slice(aes_session_key.as_ref())?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_data = cipher.encrypt(&nonce, tar_data.as_ref());

    // Safe aes key sharing
    let mut rng = thread_rng();
    let padding = Oaep::new::<Sha256>();
    let signed_key = credentials.rsa.public.encrypt(&mut rng, padding, aes_session_key.as_ref());

    // Enrure authenticity
    

    println!();
    Ok(())
}