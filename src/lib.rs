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

pub async fn parse_request(request: &str) -> Result<Request, Box<dyn std::error::Error>> {
    if request.as_bytes()[0] != b'*'{
        return Err("not a valid request".into());
    }

    let raw_vec: Vec<&str> = request.split("\r\n").collect();

    let repetitions: usize = raw_vec[0][1..].parse()?;

    let mut arguments: Vec<&str> = Vec::new();

    for i in 0..repetitions{
        arguments.push(raw_vec[2 + (i * 2)]);
    }
    
    let command: Option<Command> = match arguments[0]{
        "SET" => Some(Command::SET),
        "GET" => Some(Command::GET),
        "DELETE" => Some(Command::DELETE),
        _ => None,
    };
    
    let command = command.ok_or("invalid command")?;

    match (&command, repetitions - 1) {
        (Command::SET, value) if value != 2 => return Err("invalid command".into()),
        (Command::GET, value) if value != 1 => return Err("invalid command".into()),
        (Command::DELETE, value) if value < 1 => return Err("invalid command".into()),
        _ => {},
    }
    

    Ok(Request::new(command, arguments[1..].iter().map(|arg| arg.to_string()).collect()))
}

pub async fn read_buffer(mut stream: TcpStream) -> Result<String, Box<dyn std::error::Error>>  {
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
    
    pub async fn execute(&mut self, request: Request) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match request.command {
            Command::SET => {
                match self.cache.insert(request.arguments[0].clone(), request.arguments[1].clone()){
                    Some(value) => {
                        println!("old value was replaced with: {}: {}", 
                            request.arguments[0].clone(),
                            request.arguments[1].clone());
                        println!("{:?}", self.cache);
                        Ok(None)
                    }
                    None => {
                        println!("new value was inserted");
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
                        println!("no value was found");
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
                            println!("no value to remove");
                        }
                    }

                }
                Ok(None)
            },
        } 
    }

} 


