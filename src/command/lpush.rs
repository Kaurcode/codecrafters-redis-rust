use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::{KeyValueStore, KeyValueStoreEntry, KeyValueStoreListEntry};

pub struct LPushCommand {
    key: String,
    values: Vec<String>,
}

fn prepend(store: &mut Box<dyn KeyValueStore>, key: String, other: Vec<String>) -> Result<usize, &'static str> {
    if let Some(entry) = store.get_mut(&key) {
        return entry.prepend(other);
    }

    let mut entry: KeyValueStoreListEntry = KeyValueStoreListEntry::new();
    let return_value: Result<usize, &str> = entry.prepend(other);
    store.insert(key, Box::new(entry));
    return_value
}

impl CommandRunner for LPushCommand {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Ok(size) = prepend(store, self.key, self.values) {
            return format!(":{}\r\n", size).into_bytes()
        }
        "$-1\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for LPushCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }

        Ok(Box::new(
            LPushCommand {
                key: String::from(arguments[0]),
                values: arguments[1..].iter().rev().map(|s| s.to_string()).collect(),
            }))
    }
}
