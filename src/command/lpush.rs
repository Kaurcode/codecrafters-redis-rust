use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner};
use crate::key_value_store::{KeyValueStore, KeyValueStoreEntry, KeyValueStoreListEntry};

pub struct LPushRequest {
    key: String,
    values: Vec<String>,
}

struct LPushResponse {
    length: Result<usize, &'static str>,
}

impl CommandFactory for LPushRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }

        Ok(Box::new(
            LPushRequest {
                key: String::from(arguments[0]),
                values: arguments[1..].iter().rev().map(|s| s.to_string()).collect(),
            }))
    }
}

impl LPushResponse {
    fn new(length: Result<usize, &'static str>) -> Self {
        LPushResponse { length }
    }
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

impl DataRequester for LPushRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        Box::new(LPushResponse::new(prepend(store, self.key, self.values)))
    }
}

impl CommandRunner for LPushResponse {
    fn run(self: Box<Self>) -> Vec<u8> {
        self.length.map_or_else(
            |_| "$-1\r\n".as_bytes().to_vec(), 
            |size| format!(":{}\r\n", size).into_bytes())
    }
}
