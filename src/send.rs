use std::io::{Read, Write};
use std::process::{Command, Stdio};

use std::error::Error;

use crate::err::ListError;

pub fn send(from: Option<&str>, to: &[&str], message: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut process = Command::new("sendmail");

    process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    if let Some(from) = from {
        process.arg("-f").arg(from);
    }

    for recip in to {
        process.arg(recip);
    }

    let child = process.spawn().map_err(|why| {
        Box::new(ListError {
            message: format!("couldn't spawn sendmail: {}", why),
        })
    })?;

    child.stdin.unwrap().write_all(message).map_err(|why| {
        Box::new(ListError {
            message: format!("couldn't write to sendmail: {}", why),
        })
    })?;

    let mut s = String::new();
    child.stdout.unwrap().read_to_string(&mut s).map_err(|why| {
            Box::new(ListError {
                message: format!("couldn't read sendmail output: {}", why),
            })
    })?;

    if s.len() > 0 {
        return Err(Box::new(ListError {
            message: format!("Unexpected response from sendmail: '{}'", s),
        }))
    }

    Ok(())
}
