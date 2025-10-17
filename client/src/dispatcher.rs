// Dispatcher must decide the operations to do over the event and how to parse data.
// He must be the one who calls TCP sender (?)

use anyhow::{bail, Result};
use tokio::{
    task::JoinHandle,
    sync::mpsc::{Receiver, Sender},
};
use notify::{
    Event, 
    EventKind::{Any, Access, Create, Modify, Remove, Other}
};

type FsEventReceiver = Receiver<Event>;
// Here we will use our protocol
type NetworkSender = Sender<String>;

pub async fn spawn_dispatcher(mut rx_channel: FsEventReceiver, tx_channel: NetworkSender) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        while let Some(event) = rx_channel.recv().await {
            let packet = dispatch_fs_event(event).await?;
            tx_channel.send(packet).await?;
        };
        bail!("")
    })
}

async fn dispatch_fs_event(fs_event: Event) -> Result<String> {
    // Preprocess and turn them into a network request to sync data, or update, or create etc
    // The request will be parsed to binary inside spawn_dispatcher
    // The parsed to binary request will be sent through mpsc channel also by spawn_dispatcher
    let network_packet = match fs_event.kind {
        Any => {
            println!("DISPATCHER ANY");
            String::from("DISPATCHER ANY")
        },
        Access(access_kind) => {
            println!("DISPATCHER ACCESS: {:?}", access_kind);
            String::from(format!("DISPATCHER ACCESS: {:?}", access_kind))
        },
        Create(create_kind) => {
            println!("DISPATCHER CREATE: {:?}", create_kind);
            String::from(format!("DISPATCHER CREATE: {:?}", create_kind))
        },
        Modify(modify_kind) => {
            println!("DISPATCHER MODIFY: {:?}", modify_kind);
            String::from(format!("DISPATCHER MODIFY: {:?}", modify_kind))
        },
        Remove(remove_kind) => {
            println!("DISPATCHER MODIFY: {:?}", remove_kind);
            String::from(format!("DISPATCHER MODIFY: {:?}", remove_kind))
        },
        Other => {
            println!("DISPATCHER OTHER");
            String::from("DISPATCHER OTHER")
        }
    };
    Ok(network_packet)
}