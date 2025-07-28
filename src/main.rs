#![allow(unused_imports)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    let handles: Vec<JoinHandle<()>> = listener
        .incoming()
        .filter_map(|stream| match stream {
            Ok(stream) => {
                println!("accepted new connection");
                Some(thread::spawn(move || {
                    handle_client(stream);
                }))
            }
            Err(e) => {
                println!("error: {}", e);
                None
            }
        })
        .collect();

    handles
        .into_iter()
        .for_each(|handle: JoinHandle<()>| {
            handle.join().unwrap()
        });
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];

    while let Ok(buffer_length) = stream.read(&mut buffer) {
        if buffer_length == 0 {
            break;
        }

        stream.write_all(b"+PONG\r\n").unwrap();
    }
}
