use bincode::{self, Decode, Encode};
use anyhow::{Result, Context};

use crate::constants::{BINCODE_CONFIG};

#[derive(Debug, Encode, Decode)]
pub enum InterApplicationRequest {
    // Only the client requests
    CREATE { path: String, data: Vec<u8> },
    UPDATE { path: String, data: Vec<u8> },
    RENAME { current_path: String, new_path: String },
    DELETE(String),
    // This one is harder, which data should be passed to SYNC in order to assure
    // data synchronization between client and server
    SYNC(String),
    FETCH,
}

#[derive(Debug, Encode, Decode)]
pub enum InterApplicationResponse {
    // Only the server responds
    CREATE(bool),
    UPDATE(bool),
    RENAME(bool),
    DELETE(bool),
    SYNC(bool),
    FETCH(bool),
}



#[derive(Debug, Encode, Decode)]
pub enum NimbusProtocol {
    Request(InterApplicationRequest),
    Response(InterApplicationResponse)
}

impl NimbusProtocol {
    pub fn encode(&self) -> Result<Vec<u8>> {
        bincode::encode_to_vec(self, BINCODE_CONFIG)
            .context("External library BINCODE failed to encode_to_vec")
    }

    pub fn decode(bin_vec: &[u8]) -> Result<NimbusProtocol> {
        let (decoded_data, _len): (NimbusProtocol, usize) = bincode::decode_from_slice(bin_vec, BINCODE_CONFIG)
            .context("External library BINCODE failed to decode binary")?;
        Ok(decoded_data)
    }
}