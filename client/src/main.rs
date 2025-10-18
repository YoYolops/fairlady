mod watcher;
mod dispatcher;
mod fs_adapter;

use watcher::spawn_watcher;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc,
    task::JoinHandle
};
use core::{
    constants::TCP_SERVER_ADDR,
    Result
};

use dispatcher::spawn_dispatcher;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let (fs_event_tx, fs_event_rx) = mpsc::channel(32);
    let (network_tx, mut network_rx) = mpsc::channel(32);

    let folder_watcher_handle: JoinHandle<Result<()>> = spawn_watcher(fs_event_tx).await;
    let _dispatcher_handler: JoinHandle<Result<()>> = spawn_dispatcher(fs_event_rx, network_tx).await;

    tokio::spawn(async move {
        // Listenn and sends via tcp
        while let Some(event) = network_rx.recv().await {
            let text_event = format!("{:?}", event);
            if let Err(e) = stream.write_all(text_event.as_bytes()).await {
                println!("");
                println!("{:?}", e);
                println!("");
            };
        }
    });

    match folder_watcher_handle.await {
        Ok(_) =>
            println!("CUMULUS Client gracefully exiting without errors. This is odd since the expectation is for it to live forever"),
        Err(e) => {
            println!("{:?}", e);
            println!("CUMULUS Client gracefully exiting on error");
        }
    };
    Ok(())
}
