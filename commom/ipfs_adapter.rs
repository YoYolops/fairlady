use reqwest::{self, multipart};
use anyhow::{Result, bail};
use crate::constants::{KUBO_RPC_BASE_URL, KUBO_DATA_DEFAULT_NAME};
use crate::kubo::KuboAddResponse;

pub async fn upload_data_kubo(data: Vec<u8>) -> Result<KuboAddResponse> {
    let http_client = reqwest::Client::new();
    let part = multipart::Part::bytes(data)
        .file_name(KUBO_DATA_DEFAULT_NAME);
    let form = multipart::Form::new().part("file", part);
    
    let kubo_response = http_client
        .post(KUBO_RPC_BASE_URL)
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