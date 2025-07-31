use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::key_value_store::KeyValueStore;

pub struct LRange {
    key: String,
    start: isize,
    end: isize,
}

fn get_subslice<'a>(
    store: &'a mut Box<dyn KeyValueStore>, key: &String, start: isize, end: isize
) -> Result<&'a [String], &'static str> {
    
    if let Some(entry) = store.get(&key) {
        return match entry.get_subslice(start, end) {
            Ok(slice) => Ok(slice.unwrap_or_else(|| &[])),
            Err(s) => Err(s),
        }
    }
    Ok(&[])
}

impl CommandRunner for LRange {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8> {
        if let Ok(slice) = get_subslice(store, &self.key, self.start, self.end) {
            let body: String = slice
                .iter()
                .map(|value: &String| format!("${}\r\n{}\r\n", value.len(), value))
                .collect();
            return format!("*{}\r\n{}", slice.len(), body).into_bytes()
        }
        "$-1\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for LRange {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 3 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected exactly three arguments"));
        }
        Ok(Box::new(
            LRange {
                key: String::from(arguments[0]),
                start: arguments[1].parse().unwrap(),
                end: arguments[2].parse().unwrap(),
            }
        ))
    }
}