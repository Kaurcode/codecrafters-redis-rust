use std::io::{Error, ErrorKind};
use std::time::SystemTime;
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct GetCommand {
    key: String,
}

impl CommandRunner for GetCommand {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        match store.get(&self.key) {
            Some(entity) => {
                if let Some(expiry) = entity.get_expiry() {
                    if expiry < &SystemTime::now() {
                        store.remove(&self.key);
                        return self.run(store);
                    }
                };
                if let Ok(value) = entity.get_value() {
                    format!("${}\r\n{}\r\n", value.len(), value).into_bytes()
                } else {
                    "$-1\r\n".as_bytes().to_vec()
                }
            },
            None => "$-1\r\n".as_bytes().to_vec(),
        }
    }
}

impl CommandRunnerFactory for GetCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(GetCommand { key: String::from(arguments[0]) }))
    }
}
