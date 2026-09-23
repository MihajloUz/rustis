use core::fmt;
use std::{
    collections::HashMap, sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    sync::{Mutex, mpsc},
    net::{TcpStream, TcpListener},
};

#[derive(Debug)]
pub enum Command{
    SET,
    GET,
    DELETE,
    COMMAND,
    LIST, 
}

#[derive(Debug)]
pub struct Request{
    command: Command,
    arguments: Vec<String>,
}
impl Request{
    fn new(command: Command, arguments: Vec<String>) -> Self {
        Self{
            command,
            arguments,
        }
    }
}

pub async fn parse_request(request: &str) -> Result<Request, ServerError> {
    if request.as_bytes()[0] != b'*'{
        return Err(ServerError::InvalidCommand);
    }

    let raw_vec: Vec<&str> = request.split("\r\n").collect();

    let repetitions: usize = raw_vec[0][1..].parse().map_err(|_| ServerError::InvalidCommand)?; // remake
                                                                                            // for
                                                                                            // general
                                                                                            // io
                                                                                            // later

    let mut arguments: Vec<&str> = Vec::new();

    for i in 0..repetitions{
        arguments.push(raw_vec[2 + (i * 2)]);
    }
    
    let command: Option<Command> = match arguments[0]{
        "SET" => Some(Command::SET),
        "GET" => Some(Command::GET),
        "DELETE" => Some(Command::DELETE),
        "COMMAND" => Some(Command::COMMAND),
        "LIST" => Some(Command::LIST),
        _ => None,
    };
    
    let command = command.ok_or(ServerError::InvalidCommand)?;

    match (&command, repetitions - 1) {
        (Command::SET, value) if value != 2 => return Err(ServerError::InvalidCommand),
        (Command::GET, value) if value != 1 => return Err(ServerError::InvalidCommand),
        (Command::DELETE, value) if value < 1 => return Err(ServerError::InvalidCommand),
        (Command::LIST, value) if value != 0 => return Err(ServerError::InvalidCommand),
        _ => {},
    }
    

    Ok(Request::new(command, arguments[1..].iter().map(|arg| arg.to_string()).collect()))
}

pub async fn read_buffer(stream: &mut TcpStream) -> Result<String, ServerError>  {
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let data = String::from_utf8_lossy(&buf[..n]);
    Ok(data.to_string())
}

pub struct Execution{
    cache: HashMap<String, String>,
}
impl Execution{
    pub fn new() -> Self{
        Self{
            cache: HashMap::new(),
        }
    }
    
    pub async fn execute(&mut self, request: Request) -> Result<Option<String>, ServerError> {
        match request.command {
            Command::SET => {
                match self.cache.insert(request.arguments[0].clone(), request.arguments[1].clone()){
                    Some(value) => {
                        println!("Old value was replaced with: {}: {}", 
                            request.arguments[0].clone(),
                            request.arguments[1].clone());
                        Ok(None)
                    }
                    None => {
                        println!("New value was inserted");
                        Ok(None)
                    }
                }
            },
            Command::GET => {
                match self.cache.get(&request.arguments[0].clone()){
                    Some(value) => {
                        Ok(Some(value.to_string()))
                    }
                    None => {
                        println!("No value was found"); //make an error
                        Ok(None)
                    }
                }
            },
            Command::DELETE => {
                for argument in request.arguments {
                    match self.cache.remove(&argument){
                        Some(value) => {
                            println!("removed {}", value);
                        }
                        None => {
                            println!("no value to remove"); // make an eror
                        }
                    }

                }
                Ok(None)
            },
            Command::COMMAND => {
                println!("Received COMMAND: {:?}", request.arguments);
                Ok(None)
            }
            Command::LIST => {
                println!("CACHE: {:?}", self.cache);
                Ok(None)
            }
        } 
    }

} 

#[derive(Debug)]
pub enum ServerError{
    InvalidCommand,
    IO(std::io::Error),
}

impl fmt::Display for ServerError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self{
            ServerError::InvalidCommand => {
                write!(f, "Error: Invalid command")
            },
            ServerError::IO(e)=> {
                write!(f, "{}", e)
            },
        } 
    }
}

impl From<std::io::Error> for ServerError{
    fn from(e: std::io::Error) -> ServerError {
        ServerError::IO(e)
    }
}

impl std::error::Error for ServerError {}
