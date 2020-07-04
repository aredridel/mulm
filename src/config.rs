use serde::Deserialize;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::io::prelude::*;

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

pub trait Loadable {
    fn load(dirname: String) -> Result<List, Error>;
}

impl List {
    pub fn load(dirname: String) -> Result<List, Error> {
        let dir = Path::new(&dirname);
        let config_path = dir.join("config.toml");
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let list: List = toml::from_str(&contents)?;
        Ok(list)
    }
}
