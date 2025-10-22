mod fs_adapter;
mod workers;

use core::{Result, constants::TCP_SERVER_ADDR};
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};

use workers::{dispatcher, messenger, watcher};

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let (fs_event_tx, fs_event_rx) = mpsc::channel(32);
    let (network_tx, network_rx) = mpsc::channel(32);

    let watcher_main_task: JoinHandle<Result<()>> = watcher::spawn_watcher(fs_event_tx).await;
    let _dispatcher_main_task: JoinHandle<Result<()>> =
        dispatcher::spawn_dispatcher(fs_event_rx, network_tx).await;
    let _messenger_main_task: JoinHandle<Result<()>> =
        messenger::spawn_messenger(network_rx, stream).await;

    match watcher_main_task.await {
        Ok(_) => println!(
            "CUMULUS Client gracefully exiting without errors. This is odd since the expectation is for it to live forever"
        ),
        Err(e) => {
            println!("{:?}", e);
            println!("CUMULUS Client gracefully exiting on error");
        }
    };
    Ok(())
}
