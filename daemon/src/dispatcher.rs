use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use commom::constants::USERDATA_UPDATE_TIME_SECONDS;
use anyhow::Result;
use tokio::{sync::mpsc::Receiver, task::{self, JoinHandle}, time};
use notify::{
    Event,
    EventKind::{
        Any,
        Other,
        Access,
        Create,
        Modify,
        Remove
    }
};

async fn event_dispatcher(mut watcher_receiver: Receiver<Event>) -> Result<()> {
    // Responsible for dispatching system routines according to observed watcher events
    // It throttles events to prevent reading, encrypting, tarballing and uploading excessively: one update at most every 10s
    let update_scheduled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    while let Some(event) = watcher_receiver.recv().await {
        let was_already_scheduled = update_scheduled.swap(true, Ordering::SeqCst);
        if !was_already_scheduled {
            let update_scheduled_clone = update_scheduled.clone();
            task::spawn(async move {
                match event.kind {
                    Any => println!("ANY EVENT"),
                    Access(_) => {
                        println!("ACCESS EVENT WAITING 10s");
                        let _ = time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                        println!("ACCESS EVENT FINISH WAIT");
                    },
                    Create(_) => {
                        println!("CREATE EVENT WAITING 10s");
                        let _ = time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                        println!("CREATE EVENT FINISH WAIT");
                    },
                    Modify(_) => {
                        println!("MODIFY EVENT WAITING 10s");
                        let _ = time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                        println!("MODIFY EVENT FINISH WAIT");
                    },
                    Other => println!("OTHER EVENT"),
                    Remove(_) => {
                        println!("REMOVE EVENT WAITING 10s");
                        let _ = time::sleep(Duration::from_secs(USERDATA_UPDATE_TIME_SECONDS)).await;
                        println!("REMOVE EVENT FINISH WAIT");
                    },
                };
                update_scheduled_clone.swap(false, Ordering::SeqCst);
            });
        } else { println!("IGNORING EVENT: already scheduled") };
    };

    Ok(())
}

pub async fn spawn_dispatcher(watcher_receiver: Receiver<Event>) -> JoinHandle<Result<()>> {
    task::spawn(event_dispatcher(watcher_receiver))
}