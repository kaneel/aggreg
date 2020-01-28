use yaml_rust::{Yaml, YamlLoader};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};

pub struct Config {
    pub contents: Yaml,
}

impl Config {
    pub fn new(contents: Yaml) -> Config {
        Config { contents }
    }

    pub fn from(path: &str) -> Result<Config, Error> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        if contents.len() == 0 {
            return Err(Error::new(ErrorKind::Other, "Config file is empty"));
        }

        let docs = YamlLoader::load_from_str(&contents).unwrap();

        Ok(Config::new(docs[0].clone()))
    }
}
