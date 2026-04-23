// Create a glifo implementation for at least two cryptographic algorithms (check DLT repositories)
// Given a hard-coded file path, encrypt it and send to Kubo node via RPC
// Given an CID, retrieve a file from network and decrypt it
use glifo::encrypter::handle_credentials;
use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = handle_credentials().await {
        eprintln!("Error generating keys: {}", e);
    }
}