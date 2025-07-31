use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct LPopCommand {
    key: String,
}

impl CommandRunner for LPopCommand {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        store.get_mut(&self.key)
            .and_then(|entity| { 
                entity.len().ok().filter(|&len| 0 < len)?;
                entity.pop_front().ok() 
            })
            .map(|first_element| 
                format!("${}\r\n{}\r\n", first_element.len(), first_element)
                    .as_bytes()
                    .to_vec())
            .unwrap_or_else(|| b"$-1\r\n".to_vec())
    }
}

impl CommandRunnerFactory for LPopCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(LPopCommand { key: String::from(arguments[0]) }))
    }
}
