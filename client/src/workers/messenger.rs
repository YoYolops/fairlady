// This is a WORKER - Workers are tasks with a dedicated rx_channel

use anyhow::{Context, bail};
use core::{NimbusProtocol, AnyResult, errors::client_err::WorkerError};
use tokio::{self, io::AsyncWriteExt, net::TcpStream, sync::mpsc::Receiver, task::JoinHandle};

type NimbusReceiver = Receiver<NimbusProtocol>;

pub async fn spawn_messenger(
    mut internal_network_rx: NimbusReceiver,
    mut tcp_stream: TcpStream,
) -> JoinHandle<AnyResult<()>> {
    tokio::spawn(async move {
        // Listenn and sends via tcp
        while let Some(nimbus_protocol) = internal_network_rx.recv().await {
            if let Ok(encoded) = nimbus_protocol.encode() {
                if let Err(e) = tcp_stream.write_all(&encoded[..]).await {
                    println!("");
                    println!("Faile to write in tcp stream");
                    println!("{:?}", e);
                    println!("");
                };
            } else {
                bail!("Failed to encode incoming NimbusProtocol data")
            }
        }
        // Note that this is just an error propagation, it might never be logged.
        // It is main thread's responsability to ensure main task's health
        Err(WorkerError::ErrReceiverChannelClosed).context("Messenger exited.")?
    })
}
