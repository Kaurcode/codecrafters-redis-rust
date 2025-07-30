use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct PingCommand {}

impl CommandRunnerFactory for PingCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected no arguments"));
        }

        Ok(Box::new(PingCommand {}))
    }
}

impl CommandRunner for PingCommand {
    fn run(&self, _store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        b"+PONG\r\n".to_vec()
    }
}
