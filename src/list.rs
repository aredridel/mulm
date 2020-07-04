use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use toml;

#[derive(Debug, Eq, PartialEq)]
pub enum MailingListAction<'a> {
    Subscribe(String),
    Unsubscribe(String),
    Message(&'a [u8]),
    Reject,
}

#[derive(Deserialize, Debug)]
pub struct List {
    config: Config
}

#[derive(Deserialize, Debug)]
pub struct Config {
    name: String,
    slug: String,
    open_posting: Option<bool>,
}

#[derive(Debug)]
pub struct ListError {
    message: String
}

impl Error for ListError {
}


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
        let list: List = toml::from_str(&contents)?;
        Ok(list)
    }
    pub fn subscribe(&self, _address: String) -> Result<(), Box<dyn Error>> {
        Err(Box::new(ListError { message: "oh no!".to_string() }))
    }
    pub fn unsubscribe(&self, _address: String) -> Result<(), Box<dyn Error>> {
        Err(Box::new(ListError { message: "oh no!".to_string() }))
    }
    pub fn send(&self, _message: &[u8]) -> Result<() , Box<dyn Error>> {
        Err(Box::new(ListError { message: "oh no!".to_string() }))
    }
}
