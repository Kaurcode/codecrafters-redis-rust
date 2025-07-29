use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::time::{Duration, SystemTime};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::EnvironmentEntity;

pub struct SetCommand {
    key: String,
    value: String,
    expiry: Option<Duration>,
}

impl CommandRunner for SetCommand {
    fn run(&self, environment: &mut HashMap<String, EnvironmentEntity>) -> Vec<u8> {
        let calculated_expiry: Option<SystemTime> = match self.expiry {
            Some(duration) => Some(SystemTime::now() + duration),
            None => None,
        };

        environment.insert(
            self.key.clone(),
            EnvironmentEntity {
                value: self.value.clone(),
                expiry: calculated_expiry
            });

        "+OK\r\n".as_bytes().to_vec()
    }
}

impl CommandRunnerFactory for SetCommand {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error> {
        if arguments.len() < 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "Expected at least two arguments"));
        }

        if arguments.len() == 4 && arguments[2].eq_ignore_ascii_case("px") {
            return Ok(Box::new(
                SetCommand {
                    key: String::from(arguments[0]),
                    value: String::from(arguments[1]),
                    expiry: Some(Duration::from_millis(arguments[3].parse().unwrap())),
                }));
        }

        Ok(Box::new(SetCommand {
            key: String::from(arguments[0]),
            value: String::from(arguments[1]),
            expiry: None
        }))
    }
}

