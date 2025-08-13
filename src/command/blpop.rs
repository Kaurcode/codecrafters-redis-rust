use std::io::Write;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::sync::oneshot;
use crate::command::{CommandFactory, CommandRunner, DataRequester, Reply};
use crate::key_value_store::KeyValueStore;

pub struct BLPopRequest {
    key: String,
    timeout: Option<Duration>,
}

struct BLPopResponseBody {
    timeout: Option<Duration>,
    rx: oneshot::Receiver<String>,
}

struct BLPopResponse {
    key: String,
    body: Result<BLPopResponseBody, &'static str>,
}

impl CommandFactory for BLPopRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 2 { 
            return Err(Error::new(ErrorKind::InvalidInput, "Expected two arguments"));
        }
        
        let key: String = arguments[0].to_string();
        let timeout: u64 = arguments[1]
            .parse()
            .map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Timeout must be an unsigned integer"))?;
        let timeout: Option<Duration> = if timeout == 0 {
            None
        } else {
            Some(Duration::from_secs(timeout))
        };
        
        Ok(Box::new(BLPopRequest { key, timeout }))
    }
}

impl BLPopResponse {
    fn new_err(key: String, err: &'static str) -> Self {
        Self { key, body: Err(err) }
    }
    
    fn new(key: String, timeout: Option<Duration>, rx: oneshot::Receiver<String>) -> Self {
        Self { key, body: Ok(BLPopResponseBody { timeout, rx }) }
    }
}

impl DataRequester for BLPopRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        if let Some(entity) = store.get_mut(&self.key) {
            match entity.generate_blpop_waiter() {
                Ok(rx) => Box::new(BLPopResponse::new(self.key, self.timeout, rx)),
                Err(err) => Box::new(BLPopResponse::new_err(self.key, err)),
            }
        } else {
            Box::new(BLPopResponse::new_err(self.key, "Key not found"))
        }
    }
}

fn num_of_digits(number: usize) -> usize {
    if number == 0 {
        1
    } else {
        number.ilog10() as usize + 1
    }
}

fn encode_response(key: String, value: String) -> Vec<u8> {
    let key_length: usize = key.len();
    let value_length: usize = value.len();
    let output_capacity: usize = 
        1 + 1 + 2
        + 1 + num_of_digits(key_length) + 2
        + key_length + 2
        + 1 + num_of_digits(value_length) + 2
        + value_length + 2;
    
    let mut response: Vec<u8> = Vec::with_capacity(output_capacity);
    
    response.extend_from_slice(b"*2\r\n");
    
    response.extend_from_slice(b"$");
    write!(&mut response, "{}", key_length).unwrap();
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(key.as_bytes());
    response.extend_from_slice(b"\r\n");
    
    response.extend_from_slice(b"$");
    write!(&mut response, "{}", value_length).unwrap();
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(value.as_bytes());
    response.extend_from_slice(b"\r\n");
    
    response
}

fn nil() -> Vec<u8> { "$-1\r\n".as_bytes().to_vec() }

impl CommandRunner for BLPopResponse {
    fn run(self: Box<Self>) -> Reply {
        match self.body {
            Err(_) => Reply::Immediate(nil()),
            Ok(BLPopResponseBody { rx, timeout }) => {
                let key: String = self.key;
                Reply::Deferred(Box::pin(async move {
                    if let Some(timeout) = timeout {
                        match tokio::time::timeout(timeout, rx).await {
                            Err(_elapsed) => nil(),
                            Ok(Err(_canceled)) => nil(),
                            Ok(Ok(value)) => encode_response(key, value),
                        }
                    } else {
                        match rx.await {
                            Err(_canceled) => nil(),
                            Ok(value) => encode_response(key, value),
                        }
                    }
                }))
            },
        }
    }
}
