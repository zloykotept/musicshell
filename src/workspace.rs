#![allow(dead_code, unused_variables)]
use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    env, fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    player::{self, Player},
};

const PLAYLIST_FILE_EXT: &str = ".plist";

// Structure for saving state ==============================
#[derive(Serialize, Deserialize, Debug)]
pub struct Saver {
    pub queue: VecDeque<PathBuf>,
    pub song_index: usize,
    pub volume: f32,
    pub selected_theme: String,
}

impl Saver {
    pub fn default() -> Self {
        Saver {
            queue: VecDeque::new(),
            song_index: 0,
            volume: 0.0,
            selected_theme: String::new(),
        }
    }

    pub fn save(
        &mut self,
        player: Arc<RwLock<Player>>,
        workspace: Arc<RwLock<Workspace>>,
        save_file: &PathBuf,
    ) -> Result<()> {
        let mutex = player.read().unwrap();
        let mutex_workspace = workspace.read().unwrap();
        self.queue = mutex.queue.clone();
        self.song_index = if mutex.song_index != 0 {
            mutex.song_index - 1
        } else {
            mutex.song_index
        };
        self.volume = mutex.get_volume();
        self.selected_theme = mutex_workspace.config.selected_theme.clone();

        let encoded: Vec<u8> = bincode::serialize(&self)?;
        fs::write(save_file, encoded)?;

        Ok(())
    }

    pub fn save_playlist(
        save_dir: &PathBuf,
        name: String,
        player: Arc<RwLock<Player>>,
    ) -> Result<()> {
        let queue = player.read().unwrap().queue.clone();
        let encoded = bincode::serialize(&queue)?;

        let new_path = save_dir.join(name + PLAYLIST_FILE_EXT);
        fs::write(new_path, encoded)
            .map_err(|e| anyhow!("Problem with playlists folder:\n{}", e))?;

        Ok(())
    }

    pub fn restore(save_file: &PathBuf) -> Result<Self> {
        let encoded: Vec<u8> = fs::read(save_file)?;
        let mut decoded: Saver = bincode::deserialize(&encoded)?;

        decoded.queue.retain(|path| path.exists());

        Ok(decoded)
    }

    pub fn restore_playlists(workspace: Arc<RwLock<Workspace>>, dir: &PathBuf) -> Result<()> {
        let mutex = workspace.write().unwrap();

        for entry in dir.read_dir()? {
            if entry.is_err() {
                continue;
            }

            let entry = entry.unwrap().path();
        }

        Ok(())
    }
}

// Workspace section =======================================
#[derive(PartialEq, Clone)]
pub enum Windows {
    None,
    ThemeSelect,
    Search,
    PlaylistSave,
    Error(String),
}

pub struct Workspace {
    pub config: Config,
    pub tree: Tree,
    pub running: bool,
    pub window: Windows,
    pub stdin_buffer: String,
}

impl Workspace {
    pub fn new(config: Config) -> Result<Self> {
        let tree = Tree::new()?;

        Ok(Workspace {
            config,
            tree,
            running: true,
            window: Windows::None,
            stdin_buffer: String::new(),
        })
    }
}

// Tree section =============================================
#[derive(PartialEq, Clone)]
pub enum TreeState {
    Files,
    Queue,
    Playlists,
}

impl TreeState {
    pub fn next(&self) -> Self {
        if *self == Self::Files {
            Self::Queue
        } else if *self == Self::Queue {
            Self::Playlists
        } else {
            Self::Files
        }
    }
}

pub struct Tree {
    pub cwd: PathBuf,
    pub path_list: Vec<PathBuf>,
    pub playlists: HashMap<String, PathBuf>,
    pub selected: usize,
    pub state: TreeState,
}

impl Tree {
    pub fn new() -> Result<Self> {
        let cwd = env::current_dir()?;
        let mut path_list: Vec<PathBuf> = cwd
            .read_dir()?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();
        path_list.sort_paths();

        Ok(Tree {
            cwd,
            path_list,
            selected: 0,
            state: TreeState::Files,
            playlists: HashMap::new(),
        })
    }
}

// Sorting implementation
trait PathList {
    fn sort_paths(&mut self);
}

impl PathList for Vec<PathBuf> {
    fn sort_paths(&mut self) {
        self.sort_by(|a, b| {
            if a.is_dir() && !b.is_dir() {
                Ordering::Less
            } else if !a.is_dir() && b.is_dir() {
                Ordering::Greater
            } else {
                a.to_str().unwrap().cmp(b.to_str().unwrap())
            }
        });
    }
}
