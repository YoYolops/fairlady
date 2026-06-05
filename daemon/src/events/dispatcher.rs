// This module is dedicated to fowarding events listened by the any watcher workers

use crate::FairladyEvent::{self, CLI, FS};
use anyhow::{Context, Result};
use commom::{
    constants::{
        KUBO_DEFAULT_MFS_DESTINATION_PATH, USERDATA_UPDATE_TIME_SECONDS,
        WATCHER_REACTION_TIME_SECONDS,
    },
    database::Database,
    ipfs_adapter::{self, Metadata},
};
use glifo::{
    credentials::Credentials,
    encrypter::{self, CryptoAlgorithm},
};
use notify::{
    Event,
    // EventKind::{
    //     Any,
    //     Other,
    //     Access,
    //     Create,
    //     Modify,
    //     Remove
    // }
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{sync::mpsc::Receiver, task, time};

pub async fn event_dispatcher(
    mut event_receiver: Receiver<FairladyEvent>,
    credentials: Arc<Credentials>,
    database: Arc<Database>,
    crypto_strategy: Arc<CryptoAlgorithm>,
) -> Result<()> {
    // Responsible for dispatching system routines according to observed system events
    // It throttles fs events to prevent reading, encrypting, tarballing and uploading excessively.
    let scheduled_update: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    while let Some(event) = event_receiver.recv().await {
        let credentials_clone = credentials.clone();
        let database_clone = database.clone();
        let crypto_strategy_clone = crypto_strategy.clone();

        match event {
            CLI(user_input) => {
                spawn_cli_event_handler(
                    user_input,
                    credentials_clone,
                    database_clone,
                    crypto_strategy_clone,
                )
                .await;
            }
            FS(event) => {
                let was_already_scheduled = scheduled_update.swap(true, Ordering::Acquire);
                if !was_already_scheduled {
                    let scheduled_update_clone = scheduled_update.clone();
                    println!("SPAWNING FS EVENT HANDLER:");
                    spawn_fs_event_handler(
                        event,
                        scheduled_update_clone,
                        credentials_clone,
                        database_clone,
                        crypto_strategy_clone,
                    )
                    .await;
                }
            }
        };
    }
    Ok(())
}

async fn spawn_fs_event_handler(
    event: Event,
    scheduled_update: Arc<AtomicBool>,
    credentials: Arc<Credentials>,
    database: Arc<Database>,
    crypto_strategy: Arc<CryptoAlgorithm>,
) {
    task::spawn(async move {
        match event.kind {
            _ => {
                println!("---------- SCHEDULED STARTED ----------");
                let _ = time::sleep(Duration::from_secs(WATCHER_REACTION_TIME_SECONDS)).await;
                println!("SCHEDULED IS RUNNING");
                let _ =
                    encrypt_and_upload_system_data(&credentials, &database, &crypto_strategy).await;
                println!("SCHEDULED WAIT UPDATE TIME");
                // This is essential. When fairlady stores data, it fires an FS event that is
                // detected by the system, making it resend data to kubo, which fires another event
                // that also makes fairlady send data to kubo and so on forever.
                let _ = time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                println!("---------- SCHEDULED FINISHED ----------");
            }
        };
        scheduled_update.swap(false, Ordering::Release);
    });
}

async fn spawn_cli_event_handler(
    user_input: String,
    credentials: Arc<Credentials>,
    database: Arc<Database>,
    crypto_strategy: Arc<CryptoAlgorithm>,
) {
    tokio::spawn(async move {
        match user_input.as_ref() {
            "d" => {
                let _ =
                    decrypt_and_save_foreign_data(&credentials, &database, &crypto_strategy).await;
            }
            _ => println!("Unknown cli command"),
        };
    });
}

pub async fn decrypt_and_save_foreign_data(
    credentials: &Credentials,
    database: &Database,
    crypto_strategy: &CryptoAlgorithm,
) -> Result<()> {
    if let Some(record) = database.get_last_history_record().await? {
        let data = ipfs_adapter::download_foreign_data(&record.cid)
            .await
            .context("ERROR while downloading foreign data")?;
        let decryption_result = encrypter::decrypt_foreign_data(credentials, data, crypto_strategy)
            .await
            .context("ERROR while decrypting foreign data")?;
        match decryption_result.perf_point {
            Some(perf_point) => {
                println!("Adding decryption performance point...");
                database.add_perf_point(perf_point).await?;
            }
            None => println!("No performance point provided"),
        };
        let storage_result = encrypter::store_foreign_data(decryption_result.data)
            .await
            .context("ERROR while storing foreign data")?;

        return Ok(storage_result);
    }
    println!("No data memory");
    Ok(())
}

pub async fn encrypt_and_upload_system_data(
    system_credentials: &Credentials,
    database: &Database,
    crypto_algo: &CryptoAlgorithm,
) -> Result<()> {
    if let Ok(encryption_result) =
        encrypter::encrypt_user_data(system_credentials, crypto_algo).await
    {
        let upload_metadata: Metadata =
            ipfs_adapter::upload_data_kubo(encryption_result.data).await?;
        println!("Kubo Response: {:#?}", upload_metadata);
        println!("Linking to MFS...");
        ipfs_adapter::delete_previous_link(&format!("/{}", KUBO_DEFAULT_MFS_DESTINATION_PATH))
            .await?;
        ipfs_adapter::link_data_to_kubo_mfs(&upload_metadata.cid, &upload_metadata.name).await?;
        println!("Adding to history...");
        database
            .add_to_history(&upload_metadata.cid, &upload_metadata.timestamp_nsecs)
            .await?;
        match encryption_result.perf_point {
            Some(perf_point) => {
                println!("Adding encryption performance point...");
                database.add_perf_point(perf_point).await?;
            }
            None => println!("No performance point provided"),
        }
        println!("Done!");
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}
