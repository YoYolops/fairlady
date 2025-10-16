mod watcher;

use watcher::spawn_watcher;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use core::constants::TCP_SERVER_ADDR;
use tokio::task::JoinHandle;
use anyhow::Result;
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let (tokio_tx, mut tokio_rx) = mpsc::channel(32);
    let folder_watcher_handle: JoinHandle<Result<()>> = spawn_watcher(tokio_tx).await;

    tokio::spawn(async move {
        // Since the only way of this stop receiving messages is by file watcher thread death,
        // and that is already covered below, i'll live this one loose
        while let Some(event) = tokio_rx.recv().await {
            println!("Received event from folder watcher: {:?}", event)
        }
    });

    println!("CLIENT says hi");
    let example_message = "Hello there everyone :)".as_bytes();
    stream.write_all(example_message).await?;
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
