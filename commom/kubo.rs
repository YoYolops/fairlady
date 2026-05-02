use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KuboAddResponse {
    #[serde(rename = "Hash")]
    pub cid: String, // Hash is guaranteed to always be there

    #[serde(rename = "Mode")]
    _mode: Option<String>,

    #[serde(rename = "MtimeNsecs")]
    pub mtime_nsecs: Option<u128>,

    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Size")]
    _size: Option<String>,
}

// 1. The Root Response
#[derive(Deserialize, Debug)]
pub struct KuboMetadataResponse {
    // Wrapped in Option because if an MFS directory is completely empty, 
    // Kubo might omit the "Entries" key entirely or return null.
    #[serde(rename = "Entries")]
    pub entries: Option<Vec<KuboMetadataEntry>>,
}

// 2. The actual file data inside the array
#[derive(Deserialize, Debug)]
pub struct KuboMetadataEntry {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Hash")]
    pub cid: String,

    #[serde(rename = "MtimeNsecs")]
    pub mtime_nsecs: Option<u128>, 
}