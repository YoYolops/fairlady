use anyhow::Result;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::mpsc::Sender,
};

use crate::FairladyEvent;

pub async fn watch_cli_events(emitter_channel: Sender<FairladyEvent>) -> Result<()> {
    let stdin = io::stdin();
    let mut io_line_reader = BufReader::new(stdin).lines();
    while let Ok(Some(input)) = io_line_reader.next_line().await {
        emitter_channel.send(FairladyEvent::CLI(input)).await?;
    }
    Ok(())
}

// pub async fn clear_screen() -> Result<()> {
//     let mut stdout = io::stdout();
//     stdout.write_all(b"\x1Bc").await?;
//     stdout.flush().await?;
//     Ok(())
// }
