// Create a glifo implementation for at least two cryptographic algorithms
// IMPORTANT: Need to watch data folder to be worthy of the daemon title
mod startup;

use glifo::{
    credentials::{self, Credentials}, encrypter::{self, encrypt_user_data}
};
use commom::{
    constants::KUBO_DEFAULT_MFS_DESTINATION_PATH,
    database::{Database},
    ipfs_adapter,
    kubo::KuboAddResponse
};
use anyhow::{Result, bail, Context};
use tokio;
use startup::system_startup;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = system_startup().await?;
    let credentials = credentials::handle_credentials().await?;
    let database = Database::build(Some(pool)).await?;
    encrypt_and_upload_system_data(&credentials, &database).await?;
    decrypt_and_save_foreign_data(&credentials, &database).await?;

    Ok(())
}

async fn decrypt_and_save_foreign_data(credentials: &Credentials, database: &Database) -> Result<()> {
    if let Some(record) = database.get_last_history_record().await? {
        let data = ipfs_adapter::download_foreign_data(&record.cid).await.context("ERROR while downloading foreign data")?;
        let decrypted_data = encrypter::decrypt_foreign_data(credentials, &data).await.context("ERROR while decrypting foreign data")?;
        let storage_result = encrypter::store_foreign_data(decrypted_data).await.context("ERROR while storing foreign data")?;
        return Ok(storage_result);
    }
    bail!("ERROR: fairlady is unable to ")
}

async fn encrypt_and_upload_system_data(system_credentials: &Credentials, database: &Database) -> Result<()> {
    if let Ok(data) = encrypt_user_data(system_credentials).await {
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
        database.add_to_history(&json_response.cid).await?;
        println!("Done!");
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}