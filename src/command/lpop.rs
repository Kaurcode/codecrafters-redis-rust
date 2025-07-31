use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct LPopCommand {
    key: String,
    amount: usize,
}

impl CommandRunner for LPopCommand {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        store.get_mut(&self.key)
            .and_then(|entity| entity.pop_front(self.amount).ok())
            .filter(|values| !values.is_empty())
            .map_or_else(
                || b"$-1\r\n".to_vec(),
                |values| {
                    let body: String = values
                        .iter()
                        .map(|value| format!("${}\r\n{}\r\n", value.len(), value))
                        .collect();
                    format!("*{}\r\n{}", values.len(), body).into_bytes()
                }
            )
    }
}

impl CommandRunnerFactory for LPopCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        let argument_count = arguments.len();
        if argument_count == 1 {
            return Ok(Box::new(
                LPopCommand {
                    key: String::from(arguments[0]),
                    amount: 1 }));
        }
        if argument_count == 2 {
            return Ok(Box::new(
                LPopCommand {
                    key: String::from(arguments[0]),
                    amount: arguments[1].parse().unwrap()
                }
            ))
        }
        Err(Error::new(ErrorKind::InvalidInput, "Expected one or two arguments"))
    }
}
