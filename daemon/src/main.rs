// Create a glifo implementation for at least two cryptographic algorithms (check DLT repositories)
// Given a hard-coded file path, encrypt it and send to Kubo node via RPC
// Given an CID, retrieve a file from network and decrypt it
use glifo::encrypter::{self, encrypt_data};
use anyhow::Result;

use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let system_credentials = encrypter::handle_credentials().await?;
    if let Err(e) = encrypt_data(system_credentials).await {
        eprintln!("Error encrypting data: {}", e);
    };
    Ok(())
}