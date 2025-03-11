use std::fs::{self, File};

use anyhow::{Ok, Result};
use config::Config;
use homedir::my_home;

pub mod config;

fn main() -> Result<()> {
    let home = my_home().unwrap().expect("Can't get home directory");
    let config_path = home.join("/.config/musicshell/config");

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        File::create(&config_path)?;
    }

    let config = Config::from(&config_path);

    Ok(())
}
