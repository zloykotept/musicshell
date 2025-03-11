use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub enum Action {
    Up,
    Down,
    ChildDir,
    ParentDir,
    Exit,
}

pub struct Config {
    pub keymap_local: HashMap<String, Action>,
    pub keymap_global: HashMap<String, Action>,
}

impl Config {
    pub fn from(file: &PathBuf) -> Self {
        let parser = Parser::new(file.to_path_buf());
        let keymap_local = parser.parse_keys_local();
        let keymap_global = parser.parse_keys_global();
        Config {
            keymap_local,
            keymap_global,
        }
    }
}

pub struct Parser {
    file: PathBuf,
}

impl Parser {
    pub fn new(file: PathBuf) -> Self {
        Parser { file }
    }

    pub fn parse_keys_local(&self) -> HashMap<String, Action> {}

    pub fn parse_keys_global(&self) -> HashMap<String, Action> {}
}
