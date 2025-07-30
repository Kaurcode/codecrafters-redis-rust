use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct RPushCommand {
    key: String,
    value: String,
}

impl CommandRunner for RPushCommand {
    fn run(&self, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Ok(size) = store.push(self.key.clone(), self.value.clone()) {
            return format!(":{}\r\n", size).into_bytes()
        }
        "$-1\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for RPushCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected two arguments"));
        }

        Ok(Box::new(
            RPushCommand { 
                key: String::from(arguments[0]), 
                value: String::from(arguments[1]) }))
    }
}
