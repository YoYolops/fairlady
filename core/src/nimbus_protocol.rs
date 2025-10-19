use bincode::{self, Encode};
use anyhow::{Result, Context};

#[derive(Debug, Encode)]
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

#[derive(Debug, Encode)]
pub enum InterApplicationResponse {
    // Only the server responds
    CREATE(bool),
    UPDATE(bool),
    RENAME(bool),
    DELETE(bool),
    SYNC(bool),
    FETCH(bool),
}



#[derive(Debug, Encode)]
pub enum NimbusProtocol {
    Request(InterApplicationRequest),
    Response(InterApplicationResponse)
}

impl NimbusProtocol {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let config = bincode::config::standard()
            .with_big_endian()
            .with_variable_int_encoding();

        bincode::encode_to_vec(self, config)
            .context("EXTERNAL LIBRARY FAILURE: bincode failed to encode_to_vec")
    }
}