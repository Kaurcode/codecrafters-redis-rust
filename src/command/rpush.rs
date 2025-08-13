use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner, Reply};
use crate::key_value_store::{KeyValueStore, KeyValueStoreEntry, KeyValueStoreListEntry};

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

pub struct RPushRequest {
    key: String,
    values: Vec<String>,
}

struct RPushResult {
    length: Result<usize, &'static str>,
}

impl CommandFactory for RPushRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }

        Ok(Box::new(
            RPushRequest {
                key: String::from(arguments[0]),
                values: arguments[1..].iter().map(|s| s.to_string()).collect(),
            }))
    }
}

impl RPushResult {
    fn new(length: Result<usize, &'static str>) -> Self {
        RPushResult { length }
    }
}

impl DataRequester for RPushRequest {
    fn request(mut self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        Box::new(RPushResult::new(append(store, self.key, &mut self.values)))
    }
}

impl CommandRunner for RPushResult {
    fn run(self: Box<Self>) -> Reply {
        let reply: Vec<u8> = self.length.map_or_else(
            |_| "$-1\r\n".as_bytes().to_vec(), 
            |size| format!(":{}\r\n", size).into_bytes());
        
        Reply::Immediate(reply)
    }
}
