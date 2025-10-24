mod dispatcher;
mod messenger;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use core::AnyResult;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let (messenger_tx, mut messenger_rx) = mpsc::channel(32);
    let messenger_main_task: JoinHandle<AnyResult<()>> = messenger::spawn_messenger(messenger_tx).await;

    while let Some(_message) = messenger_rx.recv().await {
        println!("FOWARDED");
    };

    match messenger_main_task.await {
        Ok(_) => println!(
            "CUMULUS Server gracefully exiting without errors. This is odd since the expectation is for it to live forever"
        ),
        Err(e) => {
            println!("{:?}", e);
            println!("CUMULUS Server gracefully exiting on messenger error. Without messenger, we cannot work!");
        }
    };
    Ok(())
}
