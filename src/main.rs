use std::fs::{self, File};

use anyhow::{Ok, Result};
use config::{Config, Parser};
use homedir::my_home;

mod config;
mod workspace;

fn main() -> Result<()> {
    let home = my_home().unwrap().expect("Can't get home directory");
    let config_path = home.join("/.config/musicshell/config.toml");

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        File::create(&config_path)?;
    }

    let parser = Parser::new(config_path.clone())?;
    let config = Config::new(&parser)?;

    Ok(())
}
