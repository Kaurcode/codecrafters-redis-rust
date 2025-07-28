#![allow(unused_imports)]

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

async fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];

    while let Ok(buffer_length) = stream.read(&mut buffer).await {
        if buffer_length == 0 {
            break;
        }

        stream.write_all(b"+PONG\r\n").await.unwrap();
    }
}
