use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use core::constants::OBSERVED_FOLDER_PATH_STRING;
use anyhow::{bail, Context, Result};
use tokio::sync::mpsc::Sender;

pub async fn watch_folder(tokio_tx: Sender<Result<Event>>) -> Result<()> {
    let (tx_sync, rx_sync) = std::sync::mpsc::channel();
    let _ = tokio::task::spawn_blocking(move || -> Result<()> {
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
        Ok(())
    }).await??;
    Ok(())
}