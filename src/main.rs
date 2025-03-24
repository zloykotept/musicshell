#![allow(dead_code, unused_variables)]
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

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
    // getting homedir and config path
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

    // init config
    let parser = Parser::new(config_path.clone())?;
    let config = Config::new(parser)?;

    // get playlists dir unless config is moved
    let playlists_dir = config.playlists_folder.clone();

    // init ratatui
    let terminal = init();

    // pointers init
    let workspace_1 = Arc::new(RwLock::new(Workspace::new(config)?));
    let workspace_2 = Arc::clone(&workspace_1);

    let (_stream, player) = Player::new().ok_or(anyhow!("Can't establish audio output"))?;
    let player_ptr_1 = Arc::new(RwLock::new(player));
    let player_ptr_2 = Arc::clone(&player_ptr_1);
    let player_ptr_3 = Arc::clone(&player_ptr_1);

    // restore saved state
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

    // restore playlist list
    if playlists_dir.exists() {
        Saver::restore_playlists(Arc::clone(&workspace_1), &playlists_dir)?;
    }

    // run program
    thread::spawn(move || player::main_loop(player_ptr_3));
    thread::spawn(move || UI::main_loop(workspace_2, player_ptr_2, terminal));

    let action_handler_result = actions::main_loop(workspace_1, player_ptr_1, &save_file);
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
