// Dispatcher must decide the operations to do over the event and how to parse data.
// He must be the one who calls TCP sender (?)

use anyhow::{bail};
use core::{logger, Result};
use tokio::{
    task::JoinHandle,
    sync::mpsc::{Receiver, Sender},
};
use notify::{Event};
use crate::client_utils::create_request_from_event;

type FsEventReceiver = Receiver<Event>;
// Here we will use our protocol
type NetworkSender = Sender<String>;
type DispatchResult = Result<Option<String>>;

pub async fn spawn_dispatcher(mut rx_channel: FsEventReceiver, tx_channel: NetworkSender) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        while let Some(event) = rx_channel.recv().await {
            println!();
            let dispatch_value: Option<String> = dispatch_fs_event(event).await?;
            match dispatch_value {
                Some(message) => tx_channel.send(message).await?,
                None => println!("No data was sent to server for this event"),
            }
            println!();
        };
        bail!("")
    })
}

async fn dispatch_fs_event(fs_event: Event) -> DispatchResult {
    // Preprocess and turn them into a network request to sync data, or update, or create etc
    // The request will be parsed to binary inside spawn_dispatcher
    // The parsed to binary request will be sent through mpsc channel also by spawn_dispatcher
    logger::info(format!("{:?}", fs_event));
    match create_request_from_event(fs_event).await {
        Ok(protocol) => Ok(Some(format!("{:?}", protocol))),
        Err(e) => {
            logger::error(format!("{:?}", e));
            Ok(None)
        }
    }
}