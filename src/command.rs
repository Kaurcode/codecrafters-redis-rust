pub mod ping;
pub mod echo;
pub mod set;
pub mod get;
pub mod rpush;
pub mod lrange;
pub mod lpush;

use std::io::Error;
use crate::key_value_store::KeyValueStore;

pub trait CommandRunner: Send {
    fn run(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Vec<u8>;
}

pub trait CommandRunnerFactory: Sized + CommandRunner
where Self: 'static {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error>;

    fn new_command_runner(arguments: &[&str]) -> Result<Box<dyn CommandRunner>, Error> {
        Self::new(arguments).map(|box_of_self| box_of_self as Box<dyn CommandRunner>)
    }
}
