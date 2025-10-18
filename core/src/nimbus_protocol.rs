#[derive(Debug)]
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

#[derive(Debug)]
pub enum InterApplicationResponse {
    // Only the server responds
    CREATE(bool),
    UPDATE(bool),
    RENAME(bool),
    DELETE(bool),
    SYNC(bool),
    FETCH(bool),
}

#[derive(Debug)]
pub enum NimbusProtocol {
    Request(InterApplicationRequest),
    Response(InterApplicationResponse)
}
