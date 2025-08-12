pub mod ping;
pub mod echo;
pub mod set;
pub mod get;
pub mod rpush;
pub mod lrange;
pub mod lpush;
pub mod llen;
pub mod lpop;

use std::io::Error;
use crate::key_value_store::KeyValueStore;

pub trait CommandFactory: Sized + DataRequester
where Self: 'static {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error>;

    fn new_command(arguments: &[&str]) -> Result<Box<dyn DataRequester>, Error> {
        Self::new(arguments).map(|box_of_self| box_of_self as Box<dyn DataRequester>)
    }
}

pub trait CommandRunner: Send {
    fn run(self: Box<Self>) -> Vec<u8>;
}

pub trait DataRequester: Send {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner>;
}
