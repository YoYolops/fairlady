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
    let task_inner_result = tokio::spawn(watch_folder(tx_clone));

    // This if blocks the thread and next while never runs, therefore channel is never consumed
    if let Err(e) = task_inner_result.await? {
        println!("{:?}", e);
    }
    // If i move the error to above if let, the task is never joined (since is never awaited)
    // and i never catch errors (hence silently exiting when target folder to watch does not exist)
    while let Some(event) = tokio_rx.recv().await {
        println!("Received event from folder watcher: {:?}", event)
    }

    println!("CLIENT says hi");
    let example_message = "Hello there everyone :)".as_bytes();
    stream.write_all(example_message).await?;

    Ok(())
}
