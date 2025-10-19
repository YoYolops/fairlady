mod watcher;
mod dispatcher;
mod fs_adapter;
mod network_handler;

use watcher::spawn_watcher;
use tokio::{
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle
};
use core::{
    constants::TCP_SERVER_ADDR,
    Result
};

use dispatcher::spawn_dispatcher;
use network_handler::spawn_network_handler;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let (fs_event_tx, fs_event_rx) = mpsc::channel(32);
    let (network_tx, network_rx) = mpsc::channel(32);

    let folder_watcher_main_task: JoinHandle<Result<()>> = spawn_watcher(fs_event_tx).await;
    let _dispatcher_main_task: JoinHandle<Result<()>> = spawn_dispatcher(fs_event_rx, network_tx).await;
    let _network_handler_main_task : JoinHandle<Result<()>> = spawn_network_handler(network_rx, stream).await;

    match folder_watcher_main_task.await {
        Ok(_) =>
            println!("CUMULUS Client gracefully exiting without errors. This is odd since the expectation is for it to live forever"),
        Err(e) => {
            println!("{:?}", e);
            println!("CUMULUS Client gracefully exiting on error");
        }
    };
    Ok(())
}
