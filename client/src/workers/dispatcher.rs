// This is a WORKER - Workers are tasks with a dedicated rx_channel
//
// https://m.media-amazon.com/images/M/MV5BNzhhZTdjM2EtNGY3NS00MjdhLWE2ZTUtYTVkYjc3OWMyNzRlXkEyXkFqcGc@._V1_QL75_UX403_.jpg
//
// Dispatcher preprocess the fs events received from watcher
// It generates NimbusProtocol instances from relevant events,
// encrypts file data when needed and fowards those protocols
// as a bytes vec to the network handler

use crate::fs_adapter::create_request_from_event;
use anyhow::Context;
use core::{AnyResult, NimbusProtocol, errors::client_err::WorkerError, logger};
use notify::Event;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

type FsEventReceiver = Receiver<Event>;
type NetworkSender = Sender<NimbusProtocol>;
type DispatchResult = AnyResult<Option<NimbusProtocol>>;

pub async fn spawn_dispatcher(
    mut rx_channel: FsEventReceiver,
    tx_channel: NetworkSender,
) -> JoinHandle<AnyResult<()>> {
    tokio::spawn(async move {
        // Need refactor to spawn multiple tasks for each event
        while let Some(event) = rx_channel.recv().await {
            println!();
            let dispatch_value: Option<NimbusProtocol> = dispatch_fs_event(&event).await?;
            match dispatch_value {
                Some(message) => tx_channel.send(message).await?,
                None => println!("No data was sent to server for this event"),
            };
            println!();
        }
        Err(WorkerError::ErrReceiverChannelClosed).context("Dispatcher exited.")?
    })
}

async fn dispatch_fs_event(fs_event: &Event) -> DispatchResult {
    // Preprocess and turn them into a network request to sync data, or update, or create etc
    // The request will be parsed to binary inside spawn_dispatcher
    // The parsed to binary request will be sent through mpsc channel also by spawn_dispatcher
    logger::info(format!("{:?}", fs_event));
    match create_request_from_event(&fs_event).await {
        Ok(protocol) => Ok(Some(protocol)),
        Err(e) => {
            logger::error(format!("{:?}", e));
            Ok(None)
        }
    }
}
