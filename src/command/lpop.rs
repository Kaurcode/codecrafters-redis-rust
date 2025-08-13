use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner, Reply};
use crate::key_value_store::KeyValueStore;

pub struct LPopRequest {
    key: String,
    amount: Option<usize>,
}

enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

struct LPopResponse {
    response: Result<OneOrMany, &'static str>,
}

impl CommandFactory for LPopRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        let argument_count = arguments.len();
        if argument_count == 1 {
            return Ok(Box::new(
                LPopRequest {
                    key: String::from(arguments[0]),
                    amount: None }));
        }
        if argument_count == 2 {
            return Ok(Box::new(
                LPopRequest {
                    key: String::from(arguments[0]),
                    amount: Some(arguments[1].parse().map_err(|_| Error::new(
                        ErrorKind::InvalidInput, "COUNT must be an unsigned integer"
                    ))?)
                }
            ))
        }
        Err(Error::new(ErrorKind::InvalidInput, "Expected one or two arguments"))
    }
}

impl LPopResponse {
    fn new(response: Result<OneOrMany, &'static str>) -> Self {
        Self { response }
    }
}

impl DataRequester for LPopRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        let popped_values = store.get_mut(&self.key)
            .map(|entity| match self.amount {
                None => entity.pop_front().map(|value| OneOrMany::One(value)),
                Some(amount) => entity.pop_front_amount(amount).map(|values| OneOrMany::Many(values)),
            }).unwrap_or(Err("No such key"));
        
        Box::new(LPopResponse::new(popped_values))
    }
}

impl CommandRunner for LPopResponse {
    fn run(self: Box<Self>) -> Reply {
        let reply: Vec<u8> = self.response.ok().filter(|entity| match entity {
            OneOrMany::One(_) => true,
            OneOrMany::Many(values) => !values.is_empty(),
        }).map(|entity| {
            match entity {
                OneOrMany::One(value) => {
                    format!("${}\r\n{}\r\n", value.len(), value).into_bytes()
                }
                OneOrMany::Many(values) => {
                    let body: String = values
                        .iter()
                        .map(|value| format!("${}\r\n{}\r\n", value.len(), value))
                        .collect();
                    format!("*{}\r\n{}", values.len(), body).into_bytes()
                }
            }
        }).unwrap_or(b"$-1\r\n".to_vec());
        
        Reply::Immediate(reply)
    }
}
