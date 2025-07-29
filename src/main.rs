mod command;
mod key_value_store;
mod parser;

use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use crate::command::CommandRunner;
use crate::key_value_store::KeyValueStoreEntry;
use crate::parser::redis_parser;

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
    let mut key_value_store: HashMap<String, KeyValueStoreEntry> = HashMap::new();

    while let Some((command, tx)) = rx.recv().await {
        let _ = tx.send(command.run(&mut key_value_store));
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
