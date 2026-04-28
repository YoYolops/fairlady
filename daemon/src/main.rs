// Create a glifo implementation for at least two cryptographic algorithms (check DLT repositories)
// Given a hard-coded file path, encrypt it and send to Kubo node via RPC
// Given an CID, retrieve a file from network and decrypt it
use glifo::encrypter::{self, encrypt_data};
use commom::{
    ipfs_adapter,
    kubo::KuboAddResponse,
    constants::KUBO_DEFAULT_MFS_DESTINATION_PATH
};
use anyhow::{Result, bail};

use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let system_credentials = encrypter::handle_credentials().await?;
    if let Ok(data) = encrypt_data(system_credentials).await {
        let hex_string = data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(" ");
        println!("~ RAW ENCRYPTED DATA ~");
        println!("{}", hex_string);
        println!("~ END ~");
        println!();
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
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}