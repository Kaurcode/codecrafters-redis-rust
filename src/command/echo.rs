use std::io::{Error, ErrorKind};
use crate::command::{DataRequester, CommandFactory, CommandRunner, Reply};
use crate::key_value_store::KeyValueStore;

pub struct EchoCommand {
    body: String,
}

impl CommandFactory for EchoCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(EchoCommand { body: String::from(arguments[0]) }))
    }
}

impl DataRequester for EchoCommand {
    fn request(self: Box<Self>, _store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner> {
        self
    }
}

impl CommandRunner for EchoCommand {
    fn run(self: Box<Self>) -> Reply {
        Reply::Immediate(format!("${}\r\n{}\r\n", self.body.len(), self.body).into_bytes())
    }
}
