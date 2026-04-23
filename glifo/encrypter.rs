use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use p256::{
    ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
};
use rand_core::{Rng, TryRng};
use rsa::{
    pkcs8::{
        DecodePrivateKey as RsaDecodePrivateKey, DecodePublicKey as RsaDecodePublicKey,
        EncodePrivateKey as RsaEncodePrivateKey, EncodePublicKey as RsaEncodePublicKey,
    },
    Oaep, RsaPrivateKey, RsaPublicKey,
};
use sha2::{Digest, Sha256};

// ─── Key material bundle ────────────────────────────────────────────────────

/// All key material needed by both sides of the protocol.
pub struct KeyBundle {
    /// Recipient: RSA-2048 key pair (public key used to wrap the AES session key)
    pub recipient_rsa_private_pem: String,
    pub recipient_rsa_public_pem: String,

    /// Sender: ECDSA P-256 key pair (private key used to sign; public key used to verify)
    pub sender_ecdsa_private_pem: String,
    pub sender_ecdsa_public_pem: String,
}

/// Generates RSA-2048 and ECDSA P-256 key pairs, prints them to stdout, and
/// returns them in a [`KeyBundle`].
pub fn generate_keys() -> KeyBundle {
    // ── RSA-2048 (recipient) ────────────────────────────────────────────────
    let rsa_private =
        RsaPrivateKey::new(&mut Rng::unwrap_mut(), 2048).expect("failed to generate RSA private key");
    let rsa_public = RsaPublicKey::from(&rsa_private);

    let recipient_rsa_private_pem = rsa_private
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .expect("failed to encode RSA private key")
        .to_string();

    let recipient_rsa_public_pem = rsa_public
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .expect("failed to encode RSA public key");

    // ── ECDSA P-256 (sender) ───────────────────────────────────────────────
    let ecdsa_signing_key = SigningKey::random(&mut Rng::unwrap_mut());
    let ecdsa_verifying_key = VerifyingKey::from(&ecdsa_signing_key);

    let sender_ecdsa_private_pem = ecdsa_signing_key
        .to_pkcs8_pem(p256::pkcs8::LineEnding::LF)
        .expect("failed to encode ECDSA private key")
        .to_string();

    let sender_ecdsa_public_pem = ecdsa_verifying_key
        .to_public_key_pem(p256::pkcs8::LineEnding::LF)
        .expect("failed to encode ECDSA public key");

    // ── Print ──────────────────────────────────────────────────────────────
    println!("=== RECIPIENT RSA-2048 PRIVATE KEY ===\n{recipient_rsa_private_pem}");
    println!("=== RECIPIENT RSA-2048 PUBLIC KEY ===\n{recipient_rsa_public_pem}");
    println!("=== SENDER ECDSA P-256 PRIVATE KEY ===\n{sender_ecdsa_private_pem}");
    println!("=== SENDER ECDSA P-256 PUBLIC KEY ===\n{sender_ecdsa_public_pem}");

    KeyBundle {
        recipient_rsa_private_pem,
        recipient_rsa_public_pem,
        sender_ecdsa_private_pem,
        sender_ecdsa_public_pem,
    }
}

// ─── Encrypted envelope ─────────────────────────────────────────────────────

/// Everything the recipient needs to recover and authenticate the plaintext.
pub struct EncryptedEnvelope {
    /// Base64-encoded AES-256-GCM ciphertext
    pub ciphertext_b64: String,
    /// Base64-encoded GCM authentication tag (16 bytes)
    pub tag_b64: String,
    /// Base64-encoded random 96-bit nonce
    pub nonce_b64: String,
    /// Base64-encoded AES session key ciphered with the recipient's RSA public key
    pub encrypted_session_key_b64: String,
    /// Base64-encoded ECDSA P-256 signature over the SHA-256 of the plaintext
    pub signature_b64: String,
    /// Hex-encoded SHA-256 digest of the original plaintext (informational)
    pub plaintext_sha256_hex: String,
}

// ─── Encrypt ────────────────────────────────────────────────────────────────

/// Encrypts `message` and returns an [`EncryptedEnvelope`].
///
/// # Arguments
/// * `message`              – plaintext bytes to protect
/// * `recipient_rsa_public` – recipient's RSA-2048 public key (PEM)
/// * `sender_ecdsa_private` – sender's ECDSA P-256 signing key (PEM)
pub fn encrypt(
    message: &[u8],
    recipient_rsa_public_pem: &str,
    sender_ecdsa_private_pem: &str,
) -> EncryptedEnvelope {
    // 1. SHA-256 hash of the original message
    let mut hasher = Sha256::new();
    hasher.update(message);
    let plaintext_hash: [u8; 32] = hasher.finalize().into();
    let plaintext_sha256_hex = hex::encode(plaintext_hash);

    // 2. Generate a random 256-bit AES session key
    let session_key_bytes: [u8; 32] = {
        let mut k = [0u8; 32];
        Rng::try_fill_bytes(&mut k).expect("Rng failed");
        k
    };

    // 3. Encrypt the message with AES-256-GCM
    //    aes-gcm appends the 16-byte tag directly to the ciphertext buffer.
    let aes_key = Key::<Aes256Gcm>::from_slice(&session_key_bytes);
    let cipher = Aes256Gcm::new(aes_key);

    let nonce_bytes: [u8; 12] = {
        let mut n = [0u8; 12];
        Rng::try_fill_bytes(&mut n).expect("Rng failed");
        n
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext_with_tag = cipher
        .encrypt(nonce, message)
        .expect("AES-GCM encryption failed");

    // Split ciphertext and tag (tag is always the last 16 bytes)
    let tag_start = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..tag_start];
    let tag = &ciphertext_with_tag[tag_start..];

    let ciphertext_b64 = BASE64.encode(ciphertext);
    let tag_b64 = BASE64.encode(tag);
    let nonce_b64 = BASE64.encode(nonce_bytes);

    // 4. Wrap the AES session key with the recipient's RSA public key (OAEP-SHA256)
    let rsa_public = RsaPublicKey::from_public_key_pem(recipient_rsa_public_pem)
        .expect("failed to parse recipient RSA public key");

    let encrypted_session_key = rsa_public
        .encrypt(&mut Rng::unwrap_mut(), Oaep::new::<Sha256>(), &session_key_bytes)
        .expect("RSA-OAEP encryption of session key failed");

    let encrypted_session_key_b64 = BASE64.encode(&encrypted_session_key);

    // 5. Sign the plaintext hash with the sender's ECDSA private key
    let ecdsa_signing_key = SigningKey::from_pkcs8_pem(sender_ecdsa_private_pem)
        .expect("failed to parse sender ECDSA private key");

    let signature: Signature = ecdsa_signing_key.sign(&plaintext_hash);
    let signature_b64 = BASE64.encode(signature.to_der());

    EncryptedEnvelope {
        ciphertext_b64,
        tag_b64,
        nonce_b64,
        encrypted_session_key_b64,
        signature_b64,
        plaintext_sha256_hex,
    }
}

// ─── Decrypt ────────────────────────────────────────────────────────────────

/// Decrypts an [`EncryptedEnvelope`], verifies the signature and hash integrity,
/// and prints the recovered plaintext to stdout.
///
/// # Arguments
/// * `envelope`              – the encrypted package produced by [`encrypt`]
/// * `recipient_rsa_private` – recipient's RSA-2048 private key (PEM)
/// * `sender_ecdsa_public`   – sender's ECDSA P-256 verifying key (PEM)
pub fn decrypt(
    envelope: &EncryptedEnvelope,
    recipient_rsa_private_pem: &str,
    sender_ecdsa_public_pem: &str,
) {
    // 1. Unwrap the AES session key with the recipient's RSA private key
    let rsa_private = RsaPrivateKey::from_pkcs8_pem(recipient_rsa_private_pem)
        .expect("failed to parse recipient RSA private key");

    let encrypted_session_key = BASE64
        .decode(&envelope.encrypted_session_key_b64)
        .expect("failed to base64-decode encrypted session key");

    let session_key_bytes = rsa_private
        .decrypt(Oaep::new::<Sha256>(), &encrypted_session_key)
        .expect("RSA-OAEP decryption of session key failed");

    // 2. Decrypt the ciphertext with AES-256-GCM
    //    Re-attach the tag so aes-gcm can verify it during decryption.
    let nonce_bytes = BASE64
        .decode(&envelope.nonce_b64)
        .expect("failed to base64-decode nonce");
    let ciphertext = BASE64
        .decode(&envelope.ciphertext_b64)
        .expect("failed to base64-decode ciphertext");
    let tag = BASE64
        .decode(&envelope.tag_b64)
        .expect("failed to base64-decode tag");

    let mut ciphertext_with_tag = ciphertext;
    ciphertext_with_tag.extend_from_slice(&tag);

    let aes_key = Key::<Aes256Gcm>::from_slice(&session_key_bytes);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext_with_tag.as_ref())
        .expect("AES-GCM decryption failed — ciphertext may have been tampered with");

    // 3. Verify SHA-256 integrity
    let mut hasher = Sha256::new();
    hasher.update(&plaintext);
    let recovered_hash: [u8; 32] = hasher.finalize().into();
    let recovered_hash_hex = hex::encode(recovered_hash);

    assert_eq!(
        recovered_hash_hex, envelope.plaintext_sha256_hex,
        "SHA-256 integrity check failed: hash mismatch"
    );

    // 4. Verify the ECDSA signature over the recovered hash
    let ecdsa_verifying_key = VerifyingKey::from_public_key_pem(sender_ecdsa_public_pem)
        .expect("failed to parse sender ECDSA public key");

    let signature_der = BASE64
        .decode(&envelope.signature_b64)
        .expect("failed to base64-decode signature");

    let signature =
        Signature::from_der(&signature_der).expect("failed to parse ECDSA signature from DER");

    ecdsa_verifying_key
        .verify(&recovered_hash, &signature)
        .expect("ECDSA signature verification failed — message may not be from the claimed sender");

    // 5. All checks passed — print the plaintext
    println!("=== DECRYPTED MESSAGE ===");
    println!(
        "{}",
        String::from_utf8(plaintext).expect("plaintext is not valid UTF-8")
    );
    println!("SHA-256 (verified): {recovered_hash_hex}");
    println!("ECDSA signature: valid ✓");
}

// ─── Demo ───────────────────────────────────────────────────────────────────

fn main() {
    println!("─── Key generation ───────────────────────────────────────────\n");
    let keys = generate_keys();

    let message = b"Hello, this is a secret message protected with hybrid encryption!";

    println!("\n─── Encrypting ───────────────────────────────────────────────\n");
    let envelope = encrypt(
        message,
        &keys.recipient_rsa_public_pem,
        &keys.sender_ecdsa_private_pem,
    );

    println!("Ciphertext (b64) : {}", envelope.ciphertext_b64);
    println!("Tag (b64)        : {}", envelope.tag_b64);
    println!("Nonce (b64)      : {}", envelope.nonce_b64);
    println!(
        "Enc. session key : {}",
        envelope.encrypted_session_key_b64
    );
    println!("Signature (b64)  : {}", envelope.signature_b64);
    println!("Plaintext SHA256 : {}", envelope.plaintext_sha256_hex);

    println!("\n─── Decrypting ───────────────────────────────────────────────\n");
    decrypt(
        &envelope,
        &keys.recipient_rsa_private_pem,
        &keys.sender_ecdsa_public_pem,
    );
}