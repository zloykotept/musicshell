#![allow(dead_code, unused_variables)]
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use actions::Action;
use anyhow::{anyhow, Ok, Result};
use config::{Config, Parser};
use player::Player;
use ratatui::init;
use ui::UI;
use workspace::{Saver, Workspace};

mod actions;
mod config;
mod player;
mod ui;
mod workspace;

const SAMPLE_CONFIG: &[u8] = include_bytes!("../config_sample/config.toml");

fn main() -> Result<()> {
    // i fucking hate getting home directory
    let mut homedir = String::new();
    if cfg!(target_os = "windows") {
        homedir = env::var("USERPROFILE")?;
    } else if cfg!(target_os = "linux") {
        homedir = env::var("HOME")?;
    }
    let config_path: PathBuf = (homedir.clone() + "/.config/musicshell/config.toml").into();

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        let mut file = File::create(&config_path)?;
        file.write_all(SAMPLE_CONFIG)?;
        println!("Wrote sample config file at {}", config_path.display());
    }

    let parser = Parser::new(config_path.clone())?;
    let config = Config::new(&parser)?;

    let terminal = init();

    let workspace_1 = Arc::new(RwLock::new(Workspace::new(config)?));
    let workspace_2 = Arc::clone(&workspace_1);

    let (_stream, player) = Player::new().ok_or(anyhow!("Can't establish audio output"))?;
    let player_ptr_1 = Arc::new(RwLock::new(player));
    let player_ptr_2 = Arc::clone(&player_ptr_1);
    let player_ptr_3 = Arc::clone(&player_ptr_1);

    let save_file: PathBuf = (homedir + "/musicshell.dat").into();
    if save_file.exists() {
        let mut mutex = player_ptr_1.write().unwrap();
        let mut mutex_workspace = workspace_1.write().unwrap();
        let data = Saver::restore(&save_file);

        if data.is_ok() {
            let data = data?;
            mutex.queue = data.queue;
            mutex.song_index = data.song_index;
            mutex.set_volume(data.volume);
            mutex_workspace.config.selected_theme = data.selected_theme;
        } else {
            fs::remove_file(&save_file)?;
        }
    }

    thread::spawn(move || player::main_loop(player_ptr_3));
    thread::spawn(move || UI::draw_cycle(workspace_2, player_ptr_2, terminal));

    let action_handler_result = Action::action_handler(workspace_1, player_ptr_1, &save_file);
    match action_handler_result {
        std::result::Result::Ok(_) => {}
        Err(e) => {
            ratatui::restore();
            return Err(e);
        }
    }

    ratatui::restore();

    Ok(())
}
