#![allow(unused_imports)]

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((socket, _addr)) => {
                println!("accepted new connection");

                tokio::spawn(async move {
                    handle_client(socket).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

}

trait CommandRunner: Send {
    fn run(&self, environment: &mut HashMap<String, String>) -> Vec<u8>;
}

trait CommandOutputFactory: Sized + CommandRunner
where Self: 'static {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error>;

    fn new_command_output(arguments: &[&str]) -> Result<Box<dyn CommandRunner>, Error> {
        Self::new(arguments).map(|box_of_self| box_of_self as Box<dyn CommandRunner>)
    }
    
}

struct PingCommand {}

impl CommandOutputFactory for PingCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected no arguments"));
        }

        Ok(Box::new(PingCommand {}))
    }
}

impl CommandRunner for PingCommand {
    fn run(&self, _environment: &mut HashMap<String, String>) -> Vec<u8> {
        b"+PONG\r\n".to_vec()
    }
}

struct EchoCommand {
    body: String,
}

impl CommandOutputFactory for EchoCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(EchoCommand { body: String::from(arguments[0]) }))
    }
}

impl CommandRunner for EchoCommand {
    fn run(&self, _environment: &mut HashMap<String, String>) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.body.len(), self.body).into_bytes()
    }
}

struct SetCommand {
    key: String,
    value: String,
}

impl CommandRunner for SetCommand {
    fn run(&self, environment: &mut HashMap<String, String>) -> Vec<u8> {
        environment.insert(self.key.clone(), self.value.clone());
        "+OK\r\n".as_bytes().to_vec()
    }
}

impl CommandOutputFactory for SetCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }


        Ok(Box::new(SetCommand { key: String::from(arguments[0]), value: String::from(arguments[1]) }))
    }
}

struct GetCommand {
    key: String,
}

impl CommandRunner for GetCommand {
    fn run(&self, environment: &mut HashMap<String, String>) -> Vec<u8> {
        match environment.get(&self.key) {
            Some(value) => format!("${}\r\n{}\r\n", value.len(), value).into_bytes(),
            None => "$-1\r\n".as_bytes().to_vec(),
        }
    }
}

impl CommandOutputFactory for GetCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }
        
        Ok(Box::new(GetCommand { key: String::from(arguments[0]) }))
    }
}

fn redis_parser(command: &str) -> Result<Box<dyn CommandRunner>, Error> {
    let lines: Vec<&str> = command.split("\r\n").collect::<Vec<&str>>();

    let argument_count_line: &str = lines.get(0).unwrap();
    if !argument_count_line.starts_with('*') {
        return Err(Error::new(
            ErrorKind::InvalidInput, 
            format!("invalid command: \"{}\", first line should start with '*'", command)));
    }
    let mut argument_count: usize = argument_count_line[1..].parse().unwrap();
    argument_count -= 1;

    let command_length_line: &str = lines.get(1).unwrap();
    if !command_length_line.starts_with('$') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid command: \"{}\", second line should start with '$'", command)));
    }
    let command_length: usize = command_length_line[1..].parse().unwrap();
    let command: &str = lines.get(2).unwrap();
    if command_length != command.len() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid command length (in bytes) for command \"{}\"; actual length: {}, expected length: {}",
                    command, command.len(), command_length)));
    }

    let arguments: &[&str] = &lines.as_slice()[3..];
    if argument_count != arguments.len() / 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid argument count for command: \"{}\"; actual count: {}, expected count: {}",
                    command, arguments.len() / 2, argument_count)));
    }

    let mut verified_arguments: Vec<&str> = Vec::with_capacity(argument_count);
    for i in 0..argument_count {
        let argument_length_line_nr: usize = i * 2;
        let argument_line_nr: usize = i * 2 + 1;

        let argument_length_line: &str = arguments.get(argument_length_line_nr).unwrap();
        if !argument_length_line.starts_with('$') {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid argument size line in command: \"{}\", argument size line should start with '$'"
                        , command)));
        }
        let argument_length: usize = argument_length_line[1..].parse().unwrap();
        let argument: &str = arguments.get(argument_line_nr).unwrap();
        if argument_length != argument.len() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid argument length (in bytes) for argument \"{}\"; actual length: {}, expected length: {}",
                        argument, argument.len(), argument_length)));
        }

        verified_arguments.insert(i, argument);
    }

    match command.to_ascii_lowercase().as_str() {
        "ping" => PingCommand::new_command_output(&verified_arguments),
        "echo" => EchoCommand::new_command_output(&verified_arguments),
        "set" => SetCommand::new_command_output(&verified_arguments),
        "get" => GetCommand::new_command_output(&verified_arguments),
        _ => Err(Error::new(ErrorKind::InvalidInput, "invalid command")),
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut environment: HashMap<String, String> = HashMap::new();
    let mut buffer: [u8; 512] = [0; 512];

    while let Ok(buffer_length) = stream.read(&mut buffer).await {
        if buffer_length == 0 {
            break;
        }
        
        let input = match std::str::from_utf8(&buffer[..buffer_length]) {
            Ok(input) => input,
            Err(_) => {
                println!("received invalid UTF-8");
                continue;
            }
        };
        
        let command = match redis_parser(input) {
            Ok(parsed_command) => parsed_command,
            Err(e) => {
                println!("error: {}", e);
                continue;
            }
        };

        stream.write_all(&command.run(&mut environment)).await.unwrap();
    }
}
