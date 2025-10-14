use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;
use core::constants::TCP_SERVER_ADDR;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(TCP_SERVER_ADDR).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    // Return value of `Ok(0)` signifies that the remote has
                    // closed
                    Ok(0) => return,
                    Ok(n) => {
                        match std::str::from_utf8(&buf[..n]) {
                            Ok(s) => println!("RECEIVED: {}", s),
                            Err(e) => eprintln!("Received invalid UTF-8: {}", e),
                        }
                    }
                    Err(_) => {
                        // Unexpected socket error. There isn't much we can do
                        // here so just stop processing.
                        return;
                    }
                }
            }
        });
    }
}