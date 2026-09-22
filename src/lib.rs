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
pub struct Request<'a>{
    command: Command,
    arguments: Vec<&'a str>,
}
impl<'a> Request<'a>{
    fn new(command: Command, arguments: Vec<&'a str>) -> Self {
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


    Ok(Request::new(command, arguments[1..].to_vec()))
}

pub async fn read_buffer(mut stream: TcpStream) -> Result<String, Box<dyn std::error::Error>>  {
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let data = String::from_utf8_lossy(&buf[..n]);
    Ok(data.to_string())
}

pub struct Execution<'a>{
    cache: HashMap<&'a str, &'a str>,
}
impl<'a> Execution<'a>{
    pub fn new() -> Self{
        Self{
            cache: HashMap::new(),
        }
    }
    
    pub async fn execute(&mut self, request: Request<'a>) -> Result<Option<&'a str>, Box<dyn std::error::Error>> {
        match request.command {
            Command::SET => {
                match self.cache.insert(request.arguments[0], request.arguments[1]){
                    Some(value) => {
                        println!("old value was replaced with: {}: {}", 
                            request.arguments[0],
                            request.arguments[1]);
                        Ok(None)
                    }
                    None => {
                        println!("new value was inserted");
                        Ok(None)
                    }
                }
            },
            Command::GET => {
                match self.cache.get(request.arguments[0]){
                    Some(value) => {
                        Ok(Some(value))
                    }
                    None => {
                        println!("no value was found");
                        Ok(None)
                    }
                }
            },
            Command::DELETE => {
                for argument in request.arguments {
                    match self.cache.remove(argument){
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
