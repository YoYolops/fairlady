use std::fmt::format;

use reqwest::{self, multipart};
use anyhow::{Result, bail};
use crate::constants::{KUBO_RPC_BASE_URL, KUBO_DEFAULT_MFS_DESTINATION_PATH};
use crate::kubo::KuboAddResponse;

pub async fn upload_data_kubo(data: Vec<u8>) -> Result<KuboAddResponse> {
    let http_client = reqwest::Client::new();
    let part = multipart::Part::bytes(data)
        .file_name(KUBO_DEFAULT_MFS_DESTINATION_PATH);
    let form = multipart::Form::new().part("file", part);
    
    let kubo_response = http_client
        .post(format!("{}/{}", KUBO_RPC_BASE_URL, "add"))
        .multipart(form)
        .send()
        .await?;
    let status = kubo_response.status();
    let response_text = kubo_response.text().await?;
    if !status.is_success() {
        bail!("KUBO RPC ERROR ({}): {}", status, response_text);
    }
    let kubo_parsed_response_body: KuboAddResponse = serde_json::from_str::<KuboAddResponse>(&response_text)?;
    Ok(kubo_parsed_response_body)
}

pub async fn link_data_to_kubo_mfs(cid: &str, filename: &str) {
    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", KUBO_RPC_BASE_URL, "files", "cp");
    let source_path = format!("/ipfs/{}", cid);
    let mfs_destination_path = format!("/{}", filename);

    let response = http_client
        .post(url)
        .query(&[
            ("arg", &source_path),
            ("arg", &mfs_destination_path)
        ])
        .send()
        .await?;

    if !response.is_success() {
        let error_text = response.text().await?;
        bail!("Failed to link data to MFS: {}", error_text);
    };

    Ok(())
}

// Will delete already existent links with the given filename from inside the MFS.
pub async fn delete_previous_link(mfs_path: &str) -> Result<()> {
    let http_client = reqwest::Client::new();
    let url = format!("{}/{}/{}", KUBO_RPC_BASE_URL, "files", "rm");
    http_client
        .post(url)
        .query(&[
            ("arg", &mfs_path),
            ("force", &"true")
        ])
        .send()
        .await;
    Ok(())
}