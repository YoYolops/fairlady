use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt, task};

use crate::AnyResult;

pub async fn fetch_fs_data(data_path: &PathBuf)-> AnyResult<Vec<u8>> {
    // Fetches the entire of the file (EXPENSIVE AND DANGEROUSLY GREEDY)
    let mut file = File::open(data_path).await?;
    task::spawn(async move {
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer).await?;
        Ok(file_buffer)
    }).await?
}