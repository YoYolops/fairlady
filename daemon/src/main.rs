// Create a glifo implementation for at least two cryptographic algorithms
// Given a hard-coded file path, encrypt it and send to Kubo node via RPC => DONE
// Given an CID, retrieve a file from network and decrypt it
// IMPORTANT: Need to watch data folder to be worthy of the daemon title
mod startup;

use glifo::{
    credentials::{self, Credentials}, encrypter::encrypt_data
};
use commom::{
    ipfs_adapter,
    kubo::KuboAddResponse,
    constants::KUBO_DEFAULT_MFS_DESTINATION_PATH
};
use anyhow::{Result};
use tokio;
use startup::system_startup;

#[tokio::main]
async fn main() -> Result<()> {
    system_startup().await?;
    let credentials = credentials::handle_credentials().await?;
    encrypt_and_upload_system_data(credentials).await?;
    Ok(())
}

async fn encrypt_and_upload_system_data(system_credentials: Credentials) -> Result<()> {
    if let Ok(data) = encrypt_data(system_credentials).await {
        println!("SENDING TO KUBO IPFS NODE");
        let json_response: KuboAddResponse = ipfs_adapter::upload_data_kubo(data).await?;
        println!("Kubo Response: {:#?}", json_response);
        println!("Linking to MFS...");
        let filename = if let Some(name) = json_response.name {
            name
        } else {
           String::from("data.bin")
        };
        ipfs_adapter::delete_previous_link(&format!("/{}", KUBO_DEFAULT_MFS_DESTINATION_PATH)).await?;
        ipfs_adapter::link_data_to_kubo_mfs(&json_response.cid, &filename).await?;
        println!("Done!");
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}