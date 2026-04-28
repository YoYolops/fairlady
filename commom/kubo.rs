use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KuboAddResponse {
    #[serde(rename = "Bytes")]
    _bytes: Option<i64>,

    #[serde(rename = "Hash")]
    pub cid: String, // Hash is guaranteed to always be there

    #[serde(rename = "Mode")]
    _mode: Option<String>,

    #[serde(rename = "Mtime")]
    _mtime: Option<i64>,

    #[serde(rename = "MtimeNsecs")]
    _mtime_nsecs: Option<i32>,

    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Size")]
    _size: Option<String>,
}