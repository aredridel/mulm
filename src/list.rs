use fs2::FileExt;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs::{rename, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
use toml;

#[derive(Debug, Eq, PartialEq)]
pub enum MailingListAction<'a> {
    Subscribe(String),
    Unsubscribe(String),
    Message(&'a [u8]),
    Reject,
}

#[derive(Deserialize, Debug)]
struct Root {
    config: Config,
}

#[derive(Debug)]
pub struct List {
    dir: String,
    config: Config,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    name: String,
    slug: String,
    open_posting: Option<bool>,
}

#[derive(Debug)]
pub struct ListError {
    message: String,
}

impl Error for ListError {}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl List {
    pub fn load(dirname: String) -> Result<List, Box<dyn Error>> {
        let dir = Path::new(&dirname);
        let config_path = dir.join("config.toml");
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let temp: Root = toml::from_str(&contents)?;
        let list = List {
            dir: dirname,
            config: temp.config,
        };
        Ok(list)
    }

    pub fn subscribe(&self, address: String) -> Result<(), Box<dyn Error>> {
        let mut subscriptions = OpenOptions::new()
            .append(true)
            .open(Path::new(&self.dir).join("subscriptions"))?;
        subscriptions.lock_shared()?;
        subscriptions.write_fmt(format_args!("{}\n", address))?;
        subscriptions.unlock()?;
        Ok(())
    }

    pub fn unsubscribe(&self, address: String) -> Result<(), Box<dyn Error>> {
        let old_name = Path::new(&self.dir).join("subscriptions");
        let new_name = Path::new(&self.dir).join("subscriptions.new");
        let old = File::open(&old_name)?;
        let mut newsubs = File::create(&new_name)?;
        old.lock_exclusive()?;

        for maybe_line in BufReader::new(&old).lines() {
            let line = maybe_line?;
            let trimmed = line.trim();
            if trimmed != address {
                newsubs.write_fmt(format_args!("{}\n", trimmed))?;
            }
        }
        rename(new_name, old_name)?;
        old.unlock()?;
        Ok(())
    }

    pub fn send(&self, _message: &[u8]) -> Result<(), Box<dyn Error>> {
        // write message to `{sequenceNo}.msg`
        // write message to mbox `{mailingListArchiveEpoch}.mbox`
        // Lock list shared
        // Hard link list to `{sequenceNo}.list` temp file for this send
        // Stat list and write {0, byte size} as 2 64-bit ints to `{sequenceNo}.pos`
        // Spawn send process and pass it {sequenceNo}
        // Release lock
        Err(Box::new(ListError {
            message: "oh no!".to_string(),
        }))
    }

    // Internal send process
    //      Open {sequenceNo}.pos
    //      Read {startPos, endPos}
    //      Open {sequenceNo}.list
    //      Seek to {startPos}
    //      While pos < endPos
    //      Read to \n as {dest}
    //      Update {startPos} and write {sequenceNo}.pos
    //      When reaching the end of the file, delete {sequenceNo}.list and {sequenceNo}.pos
}
