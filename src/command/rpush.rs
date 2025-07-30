use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct RPushCommand {
    key: String,
    values: Vec<String>,
}

impl CommandRunner for RPushCommand {
    fn run(&self, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Ok(size) = store.append(self.key.clone(), &mut self.values.clone()) {
            return format!(":{}\r\n", size).into_bytes()
        }
        "$-1\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for RPushCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }
        
        Ok(Box::new(
            RPushCommand { 
                key: String::from(arguments[0]), 
                values: arguments[1..].iter().map(|s| s.to_string()).collect(),
            }))
    }
}
