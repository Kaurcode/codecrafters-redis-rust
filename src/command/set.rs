use std::io::{Error, ErrorKind};
use std::time::{Duration, SystemTime};
use crate::command::{DataRequester, CommandFactory, CommandRunner};
use crate::key_value_store::KeyValueStore;
use crate::KeyValueStoreStringEntry;

pub struct SetCommandRequest {
    key: String,
    value: String,
    calculated_expiry: Option<SystemTime>,
}

struct SetCommandResponse {}

impl SetCommandResponse {
    fn new() -> Self {
        SetCommandResponse {}
    }
}

impl CommandRunner for SetCommandResponse {
    fn run(self: Box<Self>) -> Vec<u8> {
        "+OK\r\n".as_bytes().to_vec()
    }
}

impl DataRequester for SetCommandRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        store.insert(
            self.key,
            Box::new(KeyValueStoreStringEntry {
                value: self.value,
                expiry: self.calculated_expiry
            }));

        Box::new(SetCommandResponse::new())
    }
}

impl CommandFactory for SetCommandRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }

        if arguments.len() == 4 && arguments[2].eq_ignore_ascii_case("px") {
            let expiry_time: Duration = Duration::from_millis(arguments[3].parse().unwrap());
            let calculated_expiry: SystemTime = SystemTime::now() + expiry_time;
            
            return Ok(Box::new(
                SetCommandRequest {
                    key: String::from(arguments[0]),
                    value: String::from(arguments[1]),
                    calculated_expiry: Some(calculated_expiry),
                }));
        }

        Ok(Box::new(SetCommandRequest {
            key: String::from(arguments[0]),
            value: String::from(arguments[1]),
            calculated_expiry: None
        }))
    }
}
