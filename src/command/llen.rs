use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner};
use crate::key_value_store::KeyValueStore;

pub struct LLenCommand {
    key: String,
}

struct LLenResponse {
    length: usize,
}

impl CommandFactory for LLenCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(LLenCommand { key: String::from(arguments[0]) }))
    }
}

impl LLenResponse {
    fn new(length: usize) -> Self {
        Self { length }
    }
}

impl DataRequester for LLenCommand {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        let length: usize = store
            .get(&self.key)
            .and_then(|entity| entity.len().ok())
            .unwrap_or(0);
        
        Box::new(LLenResponse::new(length))
    }
}

impl CommandRunner for LLenResponse {
    fn run(self: Box<Self>) -> Vec<u8> { 
        format!(":{}\r\n", self.length).into_bytes()
    }
}
