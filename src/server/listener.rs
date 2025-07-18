use std::net::{TcpListener, TcpStream};

use crate::error::types::{InternalError, InternalErrorKind};

const BIND_PORT: &str = "127.0.0.1:1999";

pub fn spawn_tcp_listener() -> Result<(), InternalError> {
    // WARNING: this function blocks the execution thread
    let listener = match TcpListener::bind(BIND_PORT) {
        Ok(tcp_listener) => tcp_listener,
        Err(_) => {
            let mut error = InternalError::new(
                InternalErrorKind::TcpConnection,
                String::from("Failed to build and bind Tcp listener"),
            );
            error.sign_stack_trace(format!("{}: Line {}", file!(), line!()));
            return Err(error);
        }
    };
    
    for stream in listener.incoming() {
        println!("Received stream of data")
    }

    Ok(())
}