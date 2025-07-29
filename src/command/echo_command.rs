use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::EnvironmentEntity;

pub struct EchoCommand {
    body: String,
}

impl CommandRunnerFactory for EchoCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected a single argument"));
        }

        Ok(Box::new(EchoCommand { body: String::from(arguments[0]) }))
    }
}

impl CommandRunner for EchoCommand {
    fn run(&self, _environment: &mut HashMap<String, EnvironmentEntity>) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.body.len(), self.body).into_bytes()
    }
}

