use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct LLenCommand {
    key: String,
}

impl CommandRunner for LLenCommand {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Some(entity) = store.get(&self.key) {
            if let Ok(length) = entity.len() {
                return format!(":{}\r\n", length).into_bytes()
            }
        }
        ":0\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for LLenCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(LLenCommand { key: String::from(arguments[0]) }))
    }
}
