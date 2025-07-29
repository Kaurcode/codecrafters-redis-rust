#![allow(unused_imports)]

mod command;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::command::echo_command::EchoCommand;
use crate::command::get_command::GetCommand;
use crate::command::ping_command::PingCommand;
use crate::command::set_command::SetCommand;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let (tx, rx) = mpsc::channel::<Msg>(100);

    tokio::spawn(async move {
        command_executor(rx).await;
    });

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((socket, _addr)) => {
                println!("accepted new connection");
                let tx_clone: mpsc::Sender<Msg> = tx.clone();

                tokio::spawn(async move {
                    handle_client(socket, tx_clone).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

}

type Msg = (Box<dyn CommandRunner + Send + 'static>, oneshot::Sender<Vec<u8>>);

async fn command_executor(mut rx: mpsc::Receiver<Msg>) {
    let mut env: HashMap<String, EnvironmentEntity> = HashMap::new();

    while let Some((command, tx)) = rx.recv().await {
        let _ = tx.send(command.run(&mut env));
    }
}

struct EnvironmentEntity {
    value: String,
    expiry: Option<SystemTime>,
}

fn parse_bulk_string<'a>(length_line: &str, content: &'a str) -> Result<&'a str, &'static str> {
    if !length_line.starts_with('$') {
        return Err("Bulk string length line must start with '$'");
    }

    let declared_len: usize = length_line[1..]
        .parse()
        .map_err(|_| "Invalid length value")?;

    if declared_len != content.len() {
        return Err("Bulk string declared length does not match content length");
    }

    Ok(content)
}

fn redis_parser(command: &str) -> Result<Box<dyn CommandRunner>, Error> {
    let lines: Vec<&str> = command.split("\r\n").collect();

    let argument_count_line: &str = lines.get(0).ok_or_else(|| {
        Error::new(ErrorKind::InvalidInput, "Missing argument count line")
    })?;

    if !argument_count_line.starts_with('*') {
        return Err(Error::new(ErrorKind::InvalidInput, "First line should start with '*'"));
    }

    let total_parts: usize = argument_count_line[1..]
        .parse()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid argument count line"))?;

    if lines.len() < 1 + total_parts * 2 {
        return Err(Error::new(ErrorKind::InvalidInput, "Incomplete command input"));
    }

    let command = parse_bulk_string(
        lines.get(1).ok_or(Error::new(ErrorKind::InvalidInput, "Missing command length line"))?,
        lines.get(2).ok_or(Error::new(ErrorKind::InvalidInput, "Missing command line"))?,
    ).map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;

    let arguments: &[&str] = &lines.as_slice()[3..];
    let argument_count: usize = total_parts - 1;
    let mut verified_arguments: Vec<&str> = Vec::with_capacity(argument_count);

    for pair in arguments.chunks_exact(2) {
        let argument = parse_bulk_string(pair[0], pair[1])
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
        verified_arguments.push(argument);
    }

    match command.to_ascii_lowercase().as_str() {
        "ping" => PingCommand::new_command_runner(&verified_arguments),
        "echo" => EchoCommand::new_command_runner(&verified_arguments),
        "set" => SetCommand::new_command_runner(&verified_arguments),
        "get" => GetCommand::new_command_runner(&verified_arguments),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown command")),
    }
}

async fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<Msg>) {
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

        let (oneshot_tx, rx): (oneshot::Sender<Vec<u8>>, oneshot::Receiver<Vec<u8>>) = oneshot::channel::<Vec<u8>>();
        tx.send((command, oneshot_tx)).await.unwrap();

        stream.write_all(&rx.await.unwrap()).await.unwrap();
    }
}
