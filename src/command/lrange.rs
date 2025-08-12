use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner};
use crate::key_value_store::KeyValueStore;

pub struct LRangeRequest {
    key: String,
    start: isize,
    end: isize,
}

struct LRangeResponse {
    subslice: Result<Vec<String>, &'static str>,
}

impl CommandFactory for LRangeRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 3 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected exactly three arguments"));
        }
        Ok(Box::new(
            LRangeRequest {
                key: String::from(arguments[0]),
                start: arguments[1].parse().unwrap(),
                end: arguments[2].parse().unwrap(),
            }
        ))
    }
}

impl LRangeResponse {
    fn new(subslice: Result<Vec<String>, &'static str>) -> Self {
        LRangeResponse { subslice }
    }
}

fn get_subslice<'a>(
    store: &'a Box<dyn KeyValueStore>, key: &String, start: isize, end: isize
) -> Result<&'a [String], &'static str> {

    if let Some(entry) = store.get(&key) {
        return match entry.get_subslice(start, end) {
            Ok(slice) => Ok(slice.unwrap_or_else(|| &[])),
            Err(s) => Err(s),
        }
    }
    Ok(&[])
}

impl DataRequester for LRangeRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        let subslice = get_subslice(store, &self.key, self.start, self.end)
            .map(|slice| slice.to_vec());
        
        Box::new(LRangeResponse::new(subslice))
    }
}

impl CommandRunner for LRangeResponse {
    fn run(self: Box<Self>) -> Vec<u8> {
        if let Ok(slice) = self.subslice {
            let body: String = slice
                .iter()
                .map(|value: &String| format!("${}\r\n{}\r\n", value.len(), value))
                .collect();
            return format!("*{}\r\n{}", slice.len(), body).into_bytes()
        }
        "$-1\r\n".as_bytes().to_vec()
    }
}
