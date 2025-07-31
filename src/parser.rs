use std::io::{Error, ErrorKind};
use crate::command::{CommandRunner, CommandRunnerFactory};
use crate::command::echo::EchoCommand;
use crate::command::get::GetCommand;
use crate::command::lpush::LPushCommand;
use crate::command::lrange::LRange;
use crate::command::ping::PingCommand;
use crate::command::rpush::RPushCommand;
use crate::command::set::SetCommand;

fn parse_bulk_string<'a>(length_line: &str, content: &'a str) -> Result<&'a str, &'static str> {
    if !length_line.starts_with('$') {
        return Err("Bulk string length line must start with '$'");
    }

    let declared_len: usize = length_line[1..]
        .parse()
        .map_err(|_| "Invalid length value")?;

    if declared_len != content.len() {
        return Err("Bulk string declared length does not match content length");
    }

    Ok(content)
}

pub fn redis_parser(command: &str) -> Result<Box<dyn CommandRunner>, Error> {
    let lines: Vec<&str> = command.split("\r\n").collect();

    let argument_count_line: &str = lines.get(0).ok_or_else(|| {
        Error::new(ErrorKind::InvalidInput, "Missing argument count line")
    })?;

    if !argument_count_line.starts_with('*') {
        return Err(Error::new(ErrorKind::InvalidInput, "First line should start with '*'"));
    }

    let total_parts: usize = argument_count_line[1..]
        .parse()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid argument count line"))?;

    if lines.len() < 1 + total_parts * 2 {
        return Err(Error::new(ErrorKind::InvalidInput, "Incomplete command input"));
    }

    let command = parse_bulk_string(
        lines.get(1).ok_or(Error::new(ErrorKind::InvalidInput, "Missing command length line"))?,
        lines.get(2).ok_or(Error::new(ErrorKind::InvalidInput, "Missing command line"))?,
    ).map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;

    let arguments: &[&str] = &lines.as_slice()[3..];
    let argument_count: usize = total_parts - 1;
    let mut verified_arguments: Vec<&str> = Vec::with_capacity(argument_count);

    for pair in arguments.chunks_exact(2) {
        let argument = parse_bulk_string(pair[0], pair[1])
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err))?;
        verified_arguments.push(argument);
    }

    match command.to_ascii_lowercase().as_str() {
        "ping" => PingCommand::new_command_runner(&verified_arguments),
        "echo" => EchoCommand::new_command_runner(&verified_arguments),
        "set" => SetCommand::new_command_runner(&verified_arguments),
        "get" => GetCommand::new_command_runner(&verified_arguments),
        "rpush" => RPushCommand::new_command_runner(&verified_arguments),
        "lpush" => LPushCommand::new_command_runner(&verified_arguments),
        "lrange" => LRange::new_command_runner(&verified_arguments),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown command")),
    }
}
