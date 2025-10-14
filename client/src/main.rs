use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use core::constants::TCP_SERVER_ADDR;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(TCP_SERVER_ADDR).await?;
    let example_message = "Hey there, hope everything went well :)".as_bytes();
    stream.write_all(example_message).await?;
    println!("Hello, world!");
    Ok(())
}
