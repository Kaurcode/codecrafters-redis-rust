use std::io::{Error, ErrorKind};
use std::time::SystemTime;
use crate::command::{DataRequester, CommandFactory, CommandRunner, Reply};
use crate::key_value_store::KeyValueStore;

pub struct GetCommandRequest {
    key: String,
    current_time: SystemTime,
}

struct GetCommandResponse {
    value: Option<String>,
}

impl CommandFactory for GetCommandRequest {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(GetCommandRequest {
            key: String::from(arguments[0]),
            current_time: SystemTime::now(),
        }))
    }
}

impl GetCommandResponse {
    fn new(value: Option<String>) -> Self {
        Self { value }
    }
}
impl DataRequester for GetCommandRequest {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        let now: SystemTime = self.current_time;
        
        let (expired, value) = store.get(&self.key)
            .map(|entity| { 
                let expired = entity.get_expiry().is_some_and(|t| t < now);
                let value = if expired { 
                    None 
                } else { 
                    entity.get_value().cloned().ok() 
                };
                (expired, value) 
            }).unwrap_or((false, None));
        
        if expired {
            store.remove(&self.key);
        }
        
        Box::new(GetCommandResponse::new(value))
    }
}

impl CommandRunner for GetCommandResponse {
    fn run(self: Box<Self>) -> Reply {
        let reply: Vec<u8> = match self.value {
            Some(value) => format!("${}\r\n{}\r\n", value.len(), value).into_bytes(),
            None => "$-1\r\n".as_bytes().to_vec(),
        };
        
        Reply::Immediate(reply)
    }
}

