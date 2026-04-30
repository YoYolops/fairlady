use anyhow::Result;
use commom::constants::SYSTEM_FOREIGN_DATA_PATH;
use tokio::fs;

pub async fn system_startup() -> Result<()> {
    fs::create_dir_all(SYSTEM_FOREIGN_DATA_PATH).await?; // ensures folder existence
    Ok(())
}