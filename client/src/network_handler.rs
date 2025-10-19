// This is a MAIN TASK - Main tasks are tasks with a dedicated rx_channel

use core::{NimbusProtocol, Result};

use anyhow::bail;
use tokio::{
    self,
    net::TcpStream,
    sync::mpsc::Receiver,
    io::AsyncWriteExt,
    task::JoinHandle,
};

type NimbusReceiver = Receiver<NimbusProtocol>;

pub async fn spawn_network_handler(mut internal_network_rx: NimbusReceiver, mut tcp_stream: TcpStream) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        // Listenn and sends via tcp
        while let Some(_event) = internal_network_rx.recv().await {
            if let Err(e) = tcp_stream.write_all("In maintainance".as_bytes()).await {
                println!("");
                println!("{:?}", e);
                println!("");
            };
        }
        // Note that this is just an error propagation, it might never be logged.
        // It is main thread's responsability to ensure main task's health
        bail!("A MAIN TASK FAILED: Network Handler task's receiver channel was closed. Network Handler task exiting")
    })
}