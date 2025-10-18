pub enum InterApplicationRequest {
    // Only the client requests
    CREATE { path: String, data: Vec<u8> },
    UPDATE { path: String, data: Vec<u8> },
    DELETE(String),
    // This one is harder, which data should be passed to SYNC in order to assure
    // data synchronization between client and server
    SYNC(String),
    FETCH,
}

pub enum InterApplicationResponse {
    // Only the server responds
    CREATE(bool),
    UPDATE(bool),
    DELETE(bool),
    SYNC(bool),
    FETCH(bool),
}

pub enum NimbusProtocol {
    Request(InterApplicationRequest),
    Response(InterApplicationResponse)
}