pub mod ping;
pub mod echo;
pub mod set;
pub mod get;
pub mod rpush;
pub mod lrange;
pub mod lpush;
pub mod llen;
pub mod lpop;
pub mod blpop;

use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use crate::key_value_store::KeyValueStore;

pub trait CommandFactory: Sized + DataRequester
where Self: 'static {
    fn new(arguments: &[&str]) -> Result<Box<Self>, Error>;

    fn new_command(arguments: &[&str]) -> Result<Box<dyn DataRequester + 'static>, Error> {
        Self::new(arguments).map(|box_of_self| box_of_self as Box<dyn DataRequester>)
    }
}

pub trait DataRequester: Send + 'static {
    fn request(self: Box<Self>, store: &mut Box<dyn KeyValueStore>) -> Box<dyn CommandRunner>;
}

type ResponseFuture = Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'static>>;

pub enum Reply {
    Immediate(Vec<u8>),
    Deferred(ResponseFuture),
}

pub trait CommandRunner: Send + 'static {
    fn run(self: Box<Self>) -> Reply;
}
