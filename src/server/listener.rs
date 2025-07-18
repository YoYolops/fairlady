use std::net::{TcpListener, TcpStream};
use crate::constants::{TCP_BIND_PORT};
use crate::types::{InternalError, InternalErrorKind};

pub fn spawn_tcp_listener() -> Result<(), InternalError> {
    // WARNING: this function blocks the execution thread
    let listener = match TcpListener::bind(TCP_BIND_PORT) {
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
        println!("Received stream of data");
    }

    Ok(())
}

// pub fn handle incoming stream