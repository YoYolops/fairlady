use anyhow::{Context, Result, bail};
use commom::constants::DATA_FOLDER_PATH;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use tokio::{sync::mpsc::Sender};

type FsEventSender = Sender<Event>;

pub async fn bridge_sync_watcher(tx_async: FsEventSender) -> Result<()> {
    // Bridges sync channel to tokio's
    let (tx_sync, rx_sync) = std::sync::mpsc::channel();
    let blocking_folder_watcher_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut watcher =
            notify::recommended_watcher(tx_sync).context("Failed to spawn notify watcher")?;

        watcher
            .watch(Path::new(DATA_FOLDER_PATH), RecursiveMode::Recursive)
            .context("Failed to watch observed folder")?;

        while let Ok(event) = rx_sync.recv() {
            match event {
                // Could recover data from channel?:
                Ok(event_data) => {
                    // If yes, send ia via async channel
                    if tx_async.blocking_send(event_data).is_err() {
                        bail!("Folder watcher failed to send event to tokio runtime")
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                    bail!("Synchronous receiver channel recovery error")
                }
            }
        }
        bail!("Messenger exited.");
    })
    .await??;
    Ok(blocking_folder_watcher_handle)
}
