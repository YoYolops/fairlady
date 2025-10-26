use bincode::{
    self,
    config::{BigEndian, Configuration},
};

// Network related constants
pub const TCP_SERVER_ADDR: &str = "localhost:1999";

// Must be dinamically decided according to OS
pub const OBSERVED_FOLDER_PATH_STRING: &str = "/home/yo/Documents/cumulus/observed";

// External crates related consts:
pub const BINCODE_CONFIG: Configuration<BigEndian> = bincode::config::standard()
    .with_big_endian()
    .with_variable_int_encoding();
