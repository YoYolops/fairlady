use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use core::{constants::OBSERVED_FOLDER_PATH_STRING, logger, Result};
use anyhow::{bail, Context};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

type FsEventSender = Sender<Event>;

pub async fn spawn_watcher(tx_async: FsEventSender) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut max_retry: u8 = 3;
        while max_retry > 0 && let Err(e) = watch_folder(tx_async.clone()).await {
            max_retry -= 1;
            logger::error(format!("{:?}", e));
            logger::info(String::from("Folder watcher thread died, retrying..."));
        };
        bail!("Folder watcher could not be spawned")
    })
}

async fn watch_folder(tx_async: FsEventSender) -> Result<()> {
    // Bridges sync channel to tokio's
    let (tx_sync, rx_sync) = std::sync::mpsc::channel();
    let blocking_folder_watcher_handle = 
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut watcher = notify::recommended_watcher(tx_sync)
                .context("Failed to spawn notify watcher")?;

            watcher
                .watch(
                    Path::new(OBSERVED_FOLDER_PATH_STRING),
                    RecursiveMode::Recursive
                )
                .context("Failed to watch observed folder")?;

            while let Ok(event) = rx_sync.recv() {
                match event { // Could recover data from channel?:
                    Ok(event_data) => { 
                        // If yes, send ia via async channel
                        if tx_async.blocking_send(event_data).is_err() {
                            bail!("Folder watcher failed to send event to tokio runtime")
                        }
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        bail!("Synchronous receiver channel recovery error")
                    },
                }
            };
            bail!("There is no senders to listen to in folder watcher")
        }).await??;
    Ok(blocking_folder_watcher_handle)
}