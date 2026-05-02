use anyhow::{Context, Result, bail};
use commom::{
    constants::{
        KUBO_DEFAULT_MFS_DESTINATION_PATH, USERDATA_UPDATE_TIME_SECONDS,
        WATCHER_REACTION_TIME_SECONDS,
    },
    database::{self, Database},
    ipfs_adapter::{self, Metadata},
};
use glifo::{
    credentials::{self, Credentials},
    encrypter,
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
use tokio::{
    sync::mpsc::Receiver,
    task::{self, JoinHandle},
    time,
};

pub async fn fs_event_dispatcher(
    mut watcher_receiver: Receiver<Event>,
    credentials: Arc<Credentials>,
    database: Arc<Database>,
) -> Result<()> {
    // Responsible for dispatching system routines according to observed watcher events
    // It throttles events to prevent reading, encrypting, tarballing and uploading excessively: one update at most every 10s
    let update_scheduled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    while let Some(event) = watcher_receiver.recv().await {
        let was_already_scheduled = update_scheduled.swap(true, Ordering::SeqCst);
        if !was_already_scheduled {
            let update_scheduled_clone = update_scheduled.clone();
            let credentials_ = credentials.clone();
            let database_ = database.clone();
            task::spawn(async move {
                match event.kind {
                    _ => {
                        let _ =
                            time::sleep(Duration::from_secs(WATCHER_REACTION_TIME_SECONDS)).await;
                        let upload_result =
                            encrypt_and_upload_system_data(&credentials_, &database_).await;
                        match upload_result {
                            Ok(_) => println!("UPLOAD SUCCESSFULL"),
                            Err(e) => println!("{}", e),
                        }
                        let _ =
                            time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                    }
                };
                update_scheduled_clone.swap(false, Ordering::SeqCst);
            });
        } else {
            println!("IGNORING EVENT: already scheduled")
        };
    }

    Ok(())
}

pub async fn decrypt_and_save_foreign_data(
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

pub async fn encrypt_and_upload_system_data(
    system_credentials: &Credentials,
    database: &Database,
) -> Result<()> {
    if let Ok(data) = encrypter::encrypt_user_data(system_credentials).await {
        let upload_metadata: Metadata = ipfs_adapter::upload_data_kubo(data).await?;
        println!("Kubo Response: {:#?}", upload_metadata);
        println!("Linking to MFS...");
        ipfs_adapter::delete_previous_link(&format!("/{}", KUBO_DEFAULT_MFS_DESTINATION_PATH))
            .await?;
        ipfs_adapter::link_data_to_kubo_mfs(&upload_metadata.cid, &upload_metadata.name).await?;
        database
            .add_to_history(&upload_metadata.cid, &upload_metadata.timestamp_nsecs)
            .await?;
        println!("Done!");
    } else {
        eprintln!("Error encrypting data");
    };
    Ok(())
}
