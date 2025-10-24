// This is a WORKER - Workers are tasks with a dedicated rx_channel
//
// The wacher module is responsible for monitoring a given folder and piping
// fs events foward. Nothing else!

use anyhow::{Context, bail};
use core::{
    AnyResult, constants::OBSERVED_FOLDER_PATH_STRING, errors::client_err::WorkerError, logger,
};
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

type FsEventSender = Sender<Event>;

pub async fn spawn_watcher(tx_async: FsEventSender) -> JoinHandle<AnyResult<()>> {
    tokio::spawn(async move {
        let mut max_retry: u8 = 3;
        while max_retry > 0
            && let Err(e) = bridge_sync_watcher(tx_async.clone()).await
        {
            max_retry -= 1;
            logger::error(format!("{:?}", e));
            logger::info(String::from("Folder watcher thread died, retrying..."));
        }
        bail!("Folder watcher could not be spawned")
    })
}

async fn bridge_sync_watcher(tx_async: FsEventSender) -> AnyResult<()> {
    // Bridges sync channel to tokio's
    let (tx_sync, rx_sync) = std::sync::mpsc::channel();
    let blocking_folder_watcher_handle = tokio::task::spawn_blocking(move || -> AnyResult<()> {
        let mut watcher =
            notify::recommended_watcher(tx_sync).context("Failed to spawn notify watcher")?;

        watcher
            .watch(
                Path::new(OBSERVED_FOLDER_PATH_STRING),
                RecursiveMode::Recursive,
            )
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
        Err(WorkerError::ErrReceiverChannelClosed).context("Messenger exited.")?
    })
    .await??;
    Ok(blocking_folder_watcher_handle)
}
