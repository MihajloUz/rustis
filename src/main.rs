use std::{
    collections::HashMap, sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{Mutex, mpsc},
};
use rustis::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let mut exec: Execution = Execution::new();

    loop{
        let (mut stream, _) = listener.accept().await?;

        let buffer: String = read_buffer(&mut stream).await?;

        let request: Request = parse_request(buffer.as_str()).await?;

        match exec.execute(request).await {
            Ok(Some(value)) => {
                stream.write_all(value.as_bytes()).await.unwrap(); // remake later 
            },
            Ok(None) => {

            },
            Err(_) => return Err("some error bruh".into()), 
        };
    }

    Ok(())
}
