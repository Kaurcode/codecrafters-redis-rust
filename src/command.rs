pub mod ping_command;
pub mod echo_command;
pub mod set_command;
pub mod get_command;

use std::collections::HashMap;
use std::io::Error;
use crate::EnvironmentEntity;

pub trait CommandRunner: Send {
    fn run(&self, environment: &mut HashMap<String, EnvironmentEntity>) -> Vec<u8>;
}

pub trait CommandRunnerFactory: Sized + CommandRunner
where Self: 'static {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error>;

    fn new_command_runner(arguments: &[&str]) -> Result<Box<dyn CommandRunner>, Error> {
        Self::new(arguments).map(|box_of_self| box_of_self as Box<dyn CommandRunner>)
    }
}
