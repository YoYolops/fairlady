// Create a glifo implementation for at least two cryptographic algorithms
// IMPORTANT: Need to watch data folder to be worthy of the daemon title
mod startup;
mod watcher;
mod dispatcher;

use std::sync::Arc;

use anyhow::{Result};
use commom::{
    database::Database,
};
use glifo::credentials;
use startup::system_startup;
use tokio::{self, sync::mpsc, task};

#[tokio::main]
async fn main() -> Result<()> {
    let pool = system_startup().await?;
    let credentials = credentials::handle_credentials().await?;
    let database = Database::build(Some(pool)).await?;

    let arc_credentials = Arc::new(credentials);
    let arc_database = Arc::new(database);

    let (watcher_transmitter, watcher_receiver) = mpsc::channel(32);
    let _ = watcher::spawn_watcher(watcher_transmitter).await; // watches userdata folder
    
    let dispatcher_credentials = arc_credentials.clone();
    let dispatcher_database = arc_database.clone();
    let watcher_receiver_task = task::spawn(async move {
        let _ = dispatcher::fs_event_dispatcher(watcher_receiver, dispatcher_credentials, dispatcher_database.clone()).await; //
    }); 

    match watcher_receiver_task.await {
        Ok(_) => println!("Fairlady daemon gracefully shutting down without errors. This is odd since the expectation is for it to live forever"),
        Err(e) => {
            println!("{:?}", e);
            println!("CUMULUS Client gracefully exiting on error");
        },
    };
    Ok(())
}