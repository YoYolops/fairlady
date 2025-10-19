// This is a MAIN TASK - Main tasks are tasks with a dedicated rx_channel
//
// https://m.media-amazon.com/images/M/MV5BNzhhZTdjM2EtNGY3NS00MjdhLWE2ZTUtYTVkYjc3OWMyNzRlXkEyXkFqcGc@._V1_QL75_UX403_.jpg
//
// Dispatcher preprocess the fs events received from watcher
// It generates NimbusProtocol instances from relevant events,
// encrypts file data when needed and fowards those protocols
// as a bytes vec to the network handler

use anyhow::{bail};
use core::{logger, NimbusProtocol, Result};
use tokio::{
    task::JoinHandle,
    sync::mpsc::{Receiver, Sender},
};
use notify::{Event};
use crate::fs_adapter::create_request_from_event;

type FsEventReceiver = Receiver<Event>;
type NetworkSender = Sender<NimbusProtocol>;
type DispatchResult = Result<Option<NimbusProtocol>>;

pub async fn spawn_dispatcher(mut rx_channel: FsEventReceiver, tx_channel: NetworkSender) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        // Need refactor to spawn multiple tasks for each event
        while let Some(event) = rx_channel.recv().await {
            println!();
            let dispatch_value: Option<NimbusProtocol> = dispatch_fs_event(&event)?;
            match dispatch_value {
                Some(message) => tx_channel.send(message).await?,
                None => println!("No data was sent to server for this event"),
            };
            println!();
        };
        bail!("A MAIN TASK FAILED: Dispatcher task's receiver channel was closed. Dispatcher task exiting")
    })
}

fn dispatch_fs_event(fs_event: &Event) -> DispatchResult {
    // Preprocess and turn them into a network request to sync data, or update, or create etc
    // The request will be parsed to binary inside spawn_dispatcher
    // The parsed to binary request will be sent through mpsc channel also by spawn_dispatcher
    logger::info(format!("{:?}", fs_event));
    match create_request_from_event(&fs_event) {
        Ok(protocol) => Ok(Some(protocol)),
        Err(e) => {
            logger::error(format!("{:?}", e));
            Ok(None)
        }
    }
}