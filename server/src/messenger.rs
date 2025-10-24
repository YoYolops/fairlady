// This is a WORKER - Workers are ever living tasks.
// Imagine them as little workers in a assembly line.
//
// The server Messenger is the complete inverse of client's
// Here, the events begin their life cycle (while client's ends them there)

use core::{AnyResult, NimbusProtocol, constants::TCP_SERVER_ADDR, logger};
use tokio::{self, io::AsyncReadExt, net::TcpListener, sync::mpsc::Sender, task::JoinHandle};

type NimbusSender = Sender<NimbusProtocol>;

pub async fn spawn_messenger(tx_channel: NimbusSender) -> JoinHandle<AnyResult<()>> {
    tokio::spawn(async move {
        let listener: TcpListener = TcpListener::bind(TCP_SERVER_ADDR).await?;

        loop {
            let (mut socket, peer_addr) = listener.accept().await?;
            logger::info(format!("New connection from: {}", peer_addr));
            let tx_channel_clone = tx_channel.clone();

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            logger::success(String::from("Remote closed, successfully exiting."));
                            return;
                        }
                        Ok(packet_size) => {
                            logger::info(format!("Received {} bytes", packet_size));
                            match NimbusProtocol::decode(&buf[..packet_size]) {
                                Ok(nimbus_data) => {
                                    logger::success(format!(
                                        "Successfully decoded packet. Forwarding: {:#?}",
                                        nimbus_data
                                    ));
                                    if let Err(e) = tx_channel_clone.send(nimbus_data).await {
                                        logger::error(format!(
                                            "Failed to send data via messenger sender channel. {:?}",
                                            e
                                        ));
                                    }
                                }
                                Err(e) => {
                                    logger::error(format!(
                                        "Failed to decode NimbusProtocol: {:?} REQUEST IGNORED",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            logger::error(format!(
                                "Failed to read data from socket: {:?} REQUEST IGNORED",
                                e
                            ));
                            return;
                        }
                    }
                }
            });
        }
    })
}
