// Create a glifo implementation for at least two cryptographic algorithms
// IMPORTANT: Need to watch data folder to be worthy of the daemon title
mod startup;
mod watcher;
mod dispatcher;

use anyhow::{Context, Result, bail};
use commom::{
    constants::KUBO_DEFAULT_MFS_DESTINATION_PATH, database::Database, ipfs_adapter::{self, UploadResponse},
};
use glifo::{
    credentials::{self, Credentials},
    encrypter::{self, encrypt_user_data},
};
use startup::system_startup;
use tokio::{self, sync::mpsc::{self, Receiver}, task};
#[tokio::main]
async fn main() -> Result<()> {
    let pool = system_startup().await?;
    let credentials = credentials::handle_credentials().await?;
    let database = Database::build(Some(pool)).await?;

    // let (watcher_transmitter, mut watcher_receiver) = mpsc::channel(32);
    // let _ = watcher::spawn_watcher(watcher_transmitter).await;
    // let watcher_receiver_task = dispatcher::spawn_dispatcher(watcher_receiver).await;
    encrypt_and_upload_system_data(&credentials, &database).await?;
    decrypt_and_save_foreign_data(&credentials, &database).await?;
    // match watcher_receiver_task.await? {
    //     Ok(_) => println!("Fairlady daemon gracefully shutting down without errors. This is odd since the expectation is for it to live forever"),
    //     Err(e) => {
    //         println!("{:?}", e);
    //         println!("CUMULUS Client gracefully exiting on error");
    //     },
    // };
    Ok(())
}

async fn decrypt_and_save_foreign_data(
    credentials: &Credentials,
    database: &Database,
) -> Result<()> {
    if let Some(record) = database.get_last_history_record().await? {
        let data = ipfs_adapter::download_foreign_data(&record.cid)
            .await
            .context("ERROR while downloading foreign data")?;
        let decrypted_data = encrypter::decrypt_foreign_data(credentials, &data)
            .await
            .context("ERROR while decrypting foreign data")?;
        let storage_result = encrypter::store_foreign_data(decrypted_data)
            .await
            .context("ERROR while storing foreign data")?;
        return Ok(storage_result);
    }
    bail!("ERROR: fairlady is unable to ")
}

async fn encrypt_and_upload_system_data(
    system_credentials: &Credentials,
    database: &Database,
) -> Result<()> {
    if let Ok(data) = encrypt_user_data(system_credentials).await {
        let upload_response: UploadResponse = ipfs_adapter::upload_data_kubo(data).await?;
        println!("Kubo Response: {:#?}", upload_response);
        println!("Linking to MFS...");
        ipfs_adapter::delete_previous_link(&format!("/{}", KUBO_DEFAULT_MFS_DESTINATION_PATH))
            .await?;
        ipfs_adapter::link_data_to_kubo_mfs(&upload_response.cid, &upload_response.name).await?;
        database.add_to_history(&upload_response.cid, &upload_response.mtime_nsecs).await?;
        println!("Done!");
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}
