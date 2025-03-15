#![allow(dead_code, unused_variables)]
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Ok, Result};
use config::{Config, Parser};
use workspace::Workspace;

mod actions;
mod config;
mod workspace;

const SAMPLE_CONFIG: &[u8] = include_bytes!("../config_sample/config.toml");

fn main() -> Result<()> {
    // i fucking hate getting home directories
    let mut config_path_str = String::new();
    if cfg!(target_os = "windows") {
        config_path_str = env::var("USERPROFILE")?;
    } else if cfg!(target_os = "linux") {
        config_path_str = env::var("HOME")?;
    }
    config_path_str += "/.config/musicshell/config.toml";
    let config_path = PathBuf::from_str(config_path_str.as_str())?;

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        let mut file = File::create(&config_path)?;
        file.write_all(SAMPLE_CONFIG)?;
        println!("Wrote sample config file at {}", config_path.display());
    }

    let parser = Parser::new(config_path.clone())?;
    let config = Config::new(&parser)?;
    let workspace = Workspace::new(config)?;

    Ok(())
}
