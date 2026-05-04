use crate::constants::USER_DATA_FOLDER_PATH;
use anyhow::Result;
use std::path::{self, PathBuf};

pub fn get_userdata_path() -> Result<PathBuf> {
    let absolute_path_buf = path::absolute(USER_DATA_FOLDER_PATH)?;
    Ok(absolute_path_buf)
}

pub fn get_latest_data_cid() -> Result<()> {
    Ok(())
}
