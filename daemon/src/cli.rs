use std::sync::Arc;

use anyhow::{Result};
use commom::database::{ Database};
use glifo::credentials::{Credentials};
use tokio::{io::{self, AsyncBufReadExt, BufReader}};

use crate::dispatcher;

pub async fn dispatch_cli_event(credentials: Arc<Credentials>, database: Arc<Database>) -> Result<()> {
    let stdin = io::stdin();
    let mut io_line_reader = BufReader::new(stdin).lines();
    while let Ok(Some(input)) = io_line_reader.next_line().await {
        match input.as_ref() {
            "d" => {
                dispatcher::decrypt_and_save_foreign_data(&credentials, &database).await?;
            },
            _ => println!("INPUT: {}", input)
        };
    };
    Ok(())
}


// pub async fn clear_screen() -> Result<()> {
//     let mut stdout = io::stdout();
//     stdout.write_all(b"\x1Bc").await?;
//     stdout.flush().await?;
//     Ok(())
// }