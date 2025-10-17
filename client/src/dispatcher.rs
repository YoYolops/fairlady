// Dispatcher must decide the operations to do over the event and how to parse data.
// He must be the one who calls TCP sender (?)

use anyhow::{bail, Result};
use tokio::{
    task::JoinHandle,
    sync::mpsc::{Receiver, Sender},
};
use notify::{
    Event, 
    EventKind::{Create, Modify, Remove}
};

type FsEventReceiver = Receiver<Event>;
// Here we will use our protocol
type NetworkSender = Sender<String>;
type DispatchResult = Result<Option<String>>;

pub async fn spawn_dispatcher(mut rx_channel: FsEventReceiver, tx_channel: NetworkSender) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        while let Some(event) = rx_channel.recv().await {
            let dispatch_value: Option<String> = dispatch_fs_event(event).await?;
            match dispatch_value {
                Some(message) => tx_channel.send(message).await?,
                None => println!("An irrelevant fs event was notified, ignoring..."),
            }
        };
        bail!("")
    })
}

async fn dispatch_fs_event(fs_event: Event) -> DispatchResult {
    // Preprocess and turn them into a network request to sync data, or update, or create etc
    // The request will be parsed to binary inside spawn_dispatcher
    // The parsed to binary request will be sent through mpsc channel also by spawn_dispatcher
    let network_packet = match fs_event.kind {
        Create(create_kind) => {
            println!("DISPATCHER CREATE: {:?}", create_kind);
            Some(
                String::from(format!("CREATE: {:?}", create_kind))
            )
        },
        Modify(modify_kind) => {
            println!("DISPATCHER MODIFY: {:?}", modify_kind);
            Some(
                String::from(format!("MODIFY: {:?}", modify_kind))
            )
        },
        Remove(remove_kind) => {
            println!("DISPATCHER MODIFY: {:?}", remove_kind);
            Some(
                String::from(format!("MODIFY: {:?}", remove_kind))
            )
        },
        _ => None,
    };
    Ok(network_packet)
}