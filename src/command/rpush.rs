use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::{KeyValueStore, KeyValueStoreEntry, KeyValueStoreListEntry};

pub struct RPushCommand {
    key: String,
    values: Vec<String>,
}

fn _push(store: &mut Box<dyn KeyValueStore>, key: String, value: String) -> Result<usize, &'static str> {
    if let Some(entry) = store.get_mut(&key) {
        return entry._push(value);
    }

    let mut entry: KeyValueStoreListEntry = KeyValueStoreListEntry::new();
    let return_value: Result<usize, &str> = entry._push(value);
    store.insert(key, Box::new(entry));
    return_value
}

fn append(store: &mut Box<dyn KeyValueStore>, key: String, other: &mut Vec<String>) -> Result<usize, &'static str> {
    if let Some(entry) = store.get_mut(&key) {
        return entry.append(other);
    }

    let mut entry: KeyValueStoreListEntry = KeyValueStoreListEntry::new();
    let return_value: Result<usize, &str> = entry.append(other);
    store.insert(key, Box::new(entry));
    return_value
}

impl CommandRunner for RPushCommand {
    fn run(mut self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Ok(size) = append(store, self.key, &mut self.values) {
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
