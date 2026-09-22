use std::{
    collections::HashMap, sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    sync::{Mutex, mpsc},
    net::{TcpStream, TcpListener},
};
use rustis::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;


    let mut exec: Execution = Execution::new();

    loop{
        let (stream, _) = listener.accept().await?;

        let buffer: String = read_buffer(stream).await?;

        let request: Request = parse_request(buffer.as_str()).await?;

        exec.execute(request).await?;
    }

    Ok(())
}
