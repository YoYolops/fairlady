// Create a glifo implementation for at least two cryptographic algorithms
mod events;
mod startup;

use anyhow::Result;
use commom::database::Database;
use glifo::credentials;
use startup::system_startup;
use std::sync::Arc;
use tokio::{self, sync::mpsc, task::JoinSet};

pub struct WorkerID {
    pub name: String,
}

pub enum FairladyEvent {
    CLI(String),
    FS(notify::Event),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create system wide needed data
    let pool = system_startup().await?;
    let credentials = credentials::handle_credentials().await?;
    let database = Database::build(Some(pool)).await?;

    // Create arcs for sharing read-only data through multiple threads
    let arc_credentials = Arc::new(credentials);
    let arc_database = Arc::new(database);

    let dispatcher_credentials = arc_credentials.clone();
    let dispatcher_database = arc_database.clone();

    // Create channels for daemon's workers communications
    let (event_transmitter, event_receiver) = mpsc::channel(32);

    // tasks configuration setup and workers thread spawning
    let mut task_set: JoinSet<Result<WorkerID>> = JoinSet::new();
    let fs_watcher_transmitter_channel = event_transmitter.clone();
    task_set.spawn(async move {
        // Task for monitoring userdata folder events
        let mut max_retry: u8 = 3;
        while max_retry > 0
            && let Err(e) =
                events::fs::bridge_sync_fs_watcher(fs_watcher_transmitter_channel.clone()).await
        {
            max_retry -= 1;
            println!("{}", format!("{:?}", e));
            println!(
                "{}",
                String::from("Folder watcher thread died, retrying...")
            );
        }
        Ok(WorkerID {
            name: String::from("FS_Watcher"),
        })
    });

    let cli_transmitter_channel = event_transmitter.clone();
    task_set.spawn(async move {
        // Task for listening to user input via terminal
        events::cli::watch_cli_events(cli_transmitter_channel).await?;
        Ok(WorkerID {
            name: String::from("CLI_Watcher"),
        })
    });

    task_set.spawn(async move {
        // Dispatches procedures according to fs watcher events
        let _ = events::dispatcher::event_dispatcher(
            event_receiver,
            dispatcher_credentials,
            dispatcher_database.clone(),
        )
        .await?;
        Ok(WorkerID {
            name: String::from("FS_Dispatcher"),
        })
    });

    // Main workers monitoring
    while let Some(join_result) = task_set.join_next().await {
        match join_result {
            // The worker finished without panicking
            Ok(worker_result) => match worker_result {
                Ok(wid) => println!(
                    "WARNING: {} exited without errors. THis is odd, since the expectation is for it to live forever",
                    wid.name
                ),
                Err(e) => eprintln!("ERROR: some worker failer with error -> {}", e),
            },
            // The worker suffered a hard panic or was cancelled by Tokio
            Err(join_error) => {
                if join_error.is_panic() {
                    eprintln!("ERROR: A worker panicked heavily! (Main thread surviving...)");
                } else if join_error.is_cancelled() {
                    eprintln!("ERROR: A worker was cancelled.");
                }
            }
        }
    }

    Ok(())
}
