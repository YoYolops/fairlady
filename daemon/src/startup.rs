use anyhow::Result;
use commom::constants::SYSTEM_DATA_FOLDER_PATH;
use tokio::fs;

pub async fn system_startup() -> Result<()> {
    fs::create_dir_all(SYSTEM_DATA_FOLDER_PATH).await?; // ensures folder existence
    Ok(())
}