use mailparse::{addrparse, MailAddr};
use std::error::Error;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use crate::err::ListError;

pub fn send(from: Option<&str>, to: &[&str], message: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut process = Command::new("sendmail");

    process.stdin(Stdio::piped()).stdout(Stdio::piped());

    if let Some(from) = from {
        let parsed = addrparse(&from)?;
        let addr = &match parsed.first().unwrap() {
            MailAddr::Single(addr) => Ok(addr),
            MailAddr::Group(_) => Err(Box::new(ListError {
                message: format!("{:?} is not a valid from address", from),
            })),
        }?;
        process.arg("-f").arg(&addr.addr);
    }

    for recip in to {
        for addr in addrparse(recip)?.to_vec() {
            match addr {
                MailAddr::Single(addr) => {
                    process.arg(addr.addr);
                }
                MailAddr::Group(group) => {
                    for addr in group.addrs {
                        process.arg(addr.addr);
                    }
                }
            }
        }
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
    child
        .stdout
        .unwrap()
        .read_to_string(&mut s)
        .map_err(|why| {
            Box::new(ListError {
                message: format!("couldn't read sendmail output: {}", why),
            })
        })?;

    if s.len() > 0 {
        return Err(Box::new(ListError {
            message: format!("Unexpected response from sendmail: '{}'", s),
        }));
    }

    Ok(())
}
