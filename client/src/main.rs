mod watcher;

use watcher::watch_folder;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use core::constants::TCP_SERVER_ADDR;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let (tokio_tx, mut tokio_rx) = mpsc::channel(32);
    let tx_clone = tokio_tx.clone();
    let folder_watcher_handle = tokio::spawn(async move {
        if let Err(e) = watch_folder(tx_clone).await {
            println!("{:?}", e);
        }
    });
    tokio::spawn(async move {
        while let Some(event) = tokio_rx.recv().await {
            println!("Received event from folder watcher: {:?}", event)
        }
    });


    println!("CLIENT says hi");
    let example_message = "Hello there everyone :)".as_bytes();
    stream.write_all(example_message).await?;
    folder_watcher_handle.await?;
    Ok(())
}
