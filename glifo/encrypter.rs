use tokio::{
    self,
    task::{self, JoinHandle}
};
use pkcs8::der::zeroize::Zeroizing;
use rsa::{
    RsaPrivateKey,
    RsaPublicKey,
    pkcs8::{
        EncodePrivateKey,
        EncodePublicKey,
        LineEnding
    }
};
use p256::{ecdsa::{SigningKey, VerifyingKey}};
use rand::{rngs::ThreadRng, thread_rng};
use anyhow::Result;

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
    ecdsa: EcdsaKeyPair
}

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

pub fn encrypt_data() -> Result<()> {
    // Encrypts all data inside ./data folder
    Ok(())
}