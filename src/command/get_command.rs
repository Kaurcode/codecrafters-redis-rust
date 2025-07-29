use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::time::SystemTime;
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::KeyValueStoreEntry;

pub struct GetCommand {
    key: String,
}

impl CommandRunner for GetCommand {
    fn run(&self, environment: &mut HashMap<String, KeyValueStoreEntry>) -> Vec<u8> {
        match environment.get(&self.key) {
            Some(entity) => {
                if let Some(expiry) = entity.expiry {
                    if expiry < SystemTime::now() {
                        environment.remove(&self.key);
                        return self.run(environment);
                    }
                };
                let value: String = entity.value.clone();
                format!("${}\r\n{}\r\n", value.len(), value).into_bytes()
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
