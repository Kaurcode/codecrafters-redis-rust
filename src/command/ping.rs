use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner};
use crate::key_value_store::KeyValueStore;

pub struct PingCommand {}

impl CommandFactory for PingCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected no arguments"));
        }

        Ok(Box::new(PingCommand {}))
    }
}

impl DataRequester for PingCommand {
    fn request(self: Box<Self>, _store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        self
    }
}

impl CommandRunner for PingCommand {
    fn run(self: Box<Self>) -> Vec<u8> {
        b"+PONG\r\n".to_vec()
    }
}