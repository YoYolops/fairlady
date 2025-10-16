use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use core::constants::OBSERVED_FOLDER_PATH_STRING;
use anyhow::{bail, Context, Result};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub async fn spawn_watcher(tokio_tx: Sender<Result<Event>>) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut max_retry: u8 = 3;
        while max_retry > 0 && let Err(e) = watch_folder(tokio_tx.clone()).await {
            max_retry -= 1;
            println!("{:?}", e);
            println!("Folder watcher thread died, retrying...");
        };
        bail!("Folder watcher could not be spawned")
    })
}

async fn watch_folder(tokio_tx: Sender<Result<Event>>) -> Result<()> {
    let (tx_sync, rx_sync) = std::sync::mpsc::channel();
    let blocking_folder_watcher_handle = 
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut watcher = notify::recommended_watcher(tx_sync)
                .context("Failed to spawn notify watcher")?;

            watcher
                .watch(Path::new(OBSERVED_FOLDER_PATH_STRING), RecursiveMode::Recursive)
                .context("Failed to watch observed folder")?;

            while let Ok(event) = rx_sync.recv() {
                // Needed to convert from notify::Result to anyhow::Result
                let converted_result_event = event.context("Failed to parse event to anyhow error format");
                if tokio_tx.blocking_send(converted_result_event).is_err() {
                    bail!("Folder watcher failed to send event to tokio runtime")
                }
            };
            bail!("There is no senders to listen to in folder watcher")
        }).await??;
    Ok(blocking_folder_watcher_handle)
}