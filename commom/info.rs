use std::path::{self, PathBuf};
use anyhow::{Result};
use crate::constants::{DATA_FOLDER_PATH};

pub fn get_userdata_path() -> Result<PathBuf> {
    let absolute_path_buf = path::absolute(DATA_FOLDER_PATH)?;
    Ok(absolute_path_buf)
}