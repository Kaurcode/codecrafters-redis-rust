mod command;
mod key_value_store;
mod parser;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use crate::command::{CommandRunner, DataRequester};
use crate::key_value_store::{InMemoryKeyValueStore, KeyValueStore, KeyValueStoreStringEntry};
use crate::parser::redis_parser;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let (tx, rx) = mpsc::channel::<Msg>(100);

    tokio::spawn(async move {
        data_manager(rx).await;
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

type Msg = (Box<dyn DataRequester + Send>, oneshot::Sender<Box<dyn CommandRunner + Send>>);

async fn data_manager(mut rx: mpsc::Receiver<Msg>) {
    let mut key_value_store: Box<dyn KeyValueStore> = Box::new(InMemoryKeyValueStore::new());

    while let Some((command, tx)) = rx.recv().await {
        let _ = tx.send(command.request(&mut key_value_store));
    }
}

async fn handle_client(mut stream: TcpStream, store_tx: mpsc::Sender<Msg>) {
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

        let (oneshot_data_tx, data_rx): 
            (oneshot::Sender<Box<dyn CommandRunner + Send>>, 
             oneshot::Receiver<Box<dyn CommandRunner + Send>>) 
            = oneshot::channel();
        
        store_tx.send((command, oneshot_data_tx)).await.unwrap();

        stream.write_all(&data_rx.await.unwrap().run()).await.unwrap();
    }
}
