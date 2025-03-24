use std::{
    cmp::Ordering,
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Ok, Result};
use bincode::config;
use crossterm::event::{self, read, KeyCode, KeyEventKind};

use crate::{
    player::Player,
    workspace::{Saver, TreeState, Windows, Workspace, PLAYLIST_FILE_EXT},
};

pub const MUSIC_EXTENSIONS: [&str; 3] = ["mp3", "wav", "ogg"];

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Action {
    None,
    Escape,
    Up,
    Down,
    ParentDir,
    Exit,
    Select,
    ToggleTreeView,
    ToggleTreeViewBack,
    ClearQueue,
    AddToQueue,
    AddAllToQueue,
    TogglePause,
    ToggleRepeat,
    Skip,
    RewindForward(usize),
    RewindBack(usize),
    VolumeDecrease(usize),
    VolumeIncrease(usize),
    SelectTheme,
    Delete,
    PlaylistSave,
}

impl Action {
    pub fn from_str(action_str: &str) -> Option<Self> {
        match action_str {
            "Escape" => Some(Action::Escape),
            "Up" => Some(Action::Up),
            "Down" => Some(Action::Down),
            "ParentDir" => Some(Action::ParentDir),
            "Exit" => Some(Action::Exit),
            "Select" => Some(Action::Select),
            "ToggleTreeView" => Some(Action::ToggleTreeView),
            "ToggleTreeViewBack" => Some(Action::ToggleTreeViewBack),
            "ClearQueue" => Some(Action::ClearQueue),
            "AddToQueue" => Some(Action::AddToQueue),
            "AddAllToQueue" => Some(Action::AddAllToQueue),
            "TogglePause" => Some(Action::TogglePause),
            "ToggleRepeat" => Some(Action::ToggleRepeat),
            "Skip" => Some(Action::Skip),
            "SelectTheme" => Some(Action::SelectTheme),
            "Delete" => Some(Action::Delete),
            "PlaylistSave" => Some(Action::PlaylistSave),
            _ => None,
        }
    }

    pub fn from_str_arg<T: Into<usize>>(action_str: &str, arg: T) -> Option<Self> {
        match action_str {
            "RewindForward" => Some(Action::RewindForward(arg.into())),
            "RewindBack" => Some(Action::RewindBack(arg.into())),
            "VolumeDecrease" => Some(Action::VolumeDecrease(arg.into())),
            "VolumeIncrease" => Some(Action::VolumeIncrease(arg.into())),
            _ => None,
        }
    }

    pub fn action_handler(
        workspace: Arc<RwLock<Workspace>>,
        player: Arc<RwLock<Player>>,
        save_file: &PathBuf,
    ) -> Result<()> {
        let config = workspace.read().unwrap().config.clone();

        loop {
            if event::poll(Duration::from_millis(100))? {
                if let event::Event::Key(key_event) = event::read()? {
                    if let Some(action) = config.keymap_local.get(&key_event) {
                        if *action == Self::Exit {
                            Saver::default().save(
                                Arc::clone(&player),
                                Arc::clone(&workspace),
                                save_file,
                            )?;

                            workspace.write().unwrap().running = false;
                            break;
                        }

                        if let Err(err) =
                            action.perform_action(Arc::clone(&workspace), Arc::clone(&player))
                        {
                            workspace.write().unwrap().window = Windows::Error(err.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn perform_action(
        &self,
        workspace: Arc<RwLock<Workspace>>,
        player: Arc<RwLock<Player>>,
    ) -> Result<()> {
        let current_window = workspace.read().unwrap().window.clone();

        match self {
            Self::Up if current_window == Windows::None => {
                let mut selected = {
                    let mutex = workspace.read().unwrap();
                    mutex.tree.selected
                };

                selected = selected.saturating_sub(1);
                workspace.write().unwrap().tree.selected = selected;
            }
            Self::Down if current_window == Windows::None => {
                let (mut selected, list_len) = {
                    let mutex = workspace.read().unwrap();
                    let list_len = if mutex.tree.state == TreeState::Files {
                        mutex.tree.path_list.len()
                    } else if mutex.tree.state == TreeState::Queue {
                        player.read().unwrap().queue.len()
                    } else {
                        mutex.tree.playlists.len()
                    };

                    (mutex.tree.selected, list_len)
                };

                if list_len != 0 && selected < list_len - 1 {
                    selected += 1;
                }
                workspace.write().unwrap().tree.selected = selected;
            }
            Self::Select if current_window == Windows::None => {
                let mutex = workspace.read().unwrap();
                if mutex.tree.state == TreeState::Files {
                    // if we are in files
                    let element = {
                        let selected = mutex.tree.selected;
                        let path_list = mutex.tree.path_list.clone();
                        mutex
                            .tree
                            .cwd
                            .join(path_list[selected].file_name().unwrap().to_str().unwrap())
                    };

                    if element.is_dir() {
                        // if we are in files and at
                        // directory
                        let mut new_list: Vec<PathBuf> = element
                            .read_dir()?
                            .filter_map(|entry| entry.ok().map(|e| e.path()))
                            .collect();
                        new_list.sort_paths();

                        drop(mutex);
                        let mut mutex = workspace.write().unwrap();
                        mutex.tree.path_list = new_list;
                        mutex.tree.selected = 0;
                        mutex.tree.cwd = element;
                        return Ok(());
                    } else if element.is_file()                      // if we are in files and at
                                                                     // a music file
                        && element.extension().is_some()
                        && MUSIC_EXTENSIONS
                            .contains(&element.extension().unwrap().to_str().unwrap())
                    {
                        let mut queue = player.read().unwrap().queue.clone();
                        queue.push_front(element);
                        player.write().unwrap().queue = queue;
                        player.write().unwrap().restart();
                        return Ok(());
                    } else {
                        // if in files and some light
                        // error
                        return Ok(());
                    }
                } else if mutex.tree.state == TreeState::Playlists {
                    let selected = {
                        let mutex = workspace.read().unwrap();
                        let selected = mutex.tree.selected;

                        mutex.tree.playlists[selected].clone()
                    };

                    player.write().unwrap().song_index = 0;
                    player.write().unwrap().clear();
                    player.write().unwrap().queue =
                        Saver::restore_playlist(&selected, &mutex.config.playlists_folder)?;
                } else {
                    // if in queue
                    let index = mutex.tree.selected;
                    player.write().unwrap().song_index = index;
                    player.write().unwrap().clear();
                }
            }
            Self::ParentDir if current_window == Windows::None => {
                let dir = {
                    let mutex = workspace.read().unwrap();

                    if mutex.tree.state != TreeState::Files {
                        return Ok(());
                    }

                    let option = mutex.tree.cwd.parent();
                    if option.is_none() {
                        return Ok(());
                    }
                    option.unwrap().to_path_buf()
                };

                let mut new_list: Vec<PathBuf> = dir
                    .read_dir()?
                    .filter_map(|entry| entry.ok().map(|e| e.path()))
                    .collect();
                new_list.sort_paths();

                let mut mutex = workspace.write().unwrap();
                mutex.tree.path_list = new_list;
                mutex.tree.selected = 0;
                mutex.tree.cwd = dir.to_path_buf();
            }
            Self::ToggleTreeView if current_window == Windows::None => {
                let mut mutex = workspace.write().unwrap();

                mutex.tree.state = mutex.tree.state.next();
                mutex.tree.selected = 0;
            }
            Self::ToggleTreeViewBack if current_window == Windows::None => {
                let mut mutex = workspace.write().unwrap();

                mutex.tree.state = mutex.tree.state.prev();
                mutex.tree.selected = 0;
            }
            Self::AddToQueue if current_window == Windows::None => {
                let (mut queue, element) = {
                    let mutex = workspace.read().unwrap();

                    if mutex.tree.state != TreeState::Files {
                        return Ok(());
                    }

                    let selected = mutex.tree.selected;
                    let element = mutex.tree.path_list[selected].clone();
                    (player.read().unwrap().queue.clone(), element)
                };

                if !element.is_file()
                    || element.extension().is_none()
                    || !MUSIC_EXTENSIONS.contains(&element.extension().unwrap().to_str().unwrap())
                {
                    return Ok(());
                }

                queue.push_back(element);
                player.write().unwrap().queue = queue;
            }
            Self::ClearQueue => {
                player.write().unwrap().queue = VecDeque::new();
                player.write().unwrap().restart();
            }
            Self::Delete if current_window == Windows::None => {
                let tree_state = workspace.read().unwrap().tree.state.clone();

                if tree_state == TreeState::Queue {
                    let (mut queue, selected) = {
                        let mutex = workspace.read().unwrap();

                        (player.read().unwrap().queue.clone(), mutex.tree.selected)
                    };

                    if queue.is_empty() {
                        return Ok(());
                    }
                    queue.remove(selected);
                    player.write().unwrap().queue = queue;
                } else if tree_state == TreeState::Playlists {
                    let (selected, playlists_dir) = {
                        let mutex = workspace.read().unwrap();
                        let selected = mutex.tree.selected;
                        (
                            mutex.tree.playlists[selected].clone(),
                            mutex.config.playlists_folder.clone(),
                        )
                    };

                    fs::remove_file(playlists_dir.join(selected + "." + PLAYLIST_FILE_EXT))?;
                    Saver::restore_playlists(Arc::clone(&workspace), &playlists_dir)?;
                }
            }
            Self::Skip => {
                let mut mutex = player.write().unwrap();

                mutex.clear();
            }
            Self::VolumeDecrease(x) => {
                player.read().unwrap().decrease_volume(*x as f32);
            }
            Self::VolumeIncrease(x) => {
                player.read().unwrap().increase_volume(*x as f32);
            }
            Self::TogglePause => {
                let mutex = player.read().unwrap();
                if mutex.is_paused() {
                    mutex.resume();
                } else {
                    mutex.pause();
                }
            }
            Self::RewindForward(x) => {
                player.read().unwrap().rewind_forward(*x as u64);
            }
            Self::RewindBack(x) => {
                player.read().unwrap().rewind_back(*x as u64);
            }
            Self::ToggleRepeat => {
                let repeat = player.read().unwrap().repeat;
                player.write().unwrap().repeat = !repeat;
            }
            Self::Escape => {
                workspace.write().unwrap().window = Windows::None;
            }

            // theme selection Window
            Self::SelectTheme if current_window == Windows::None => {
                workspace.write().unwrap().window = Windows::ThemeSelect;
                workspace.write().unwrap().tree.selected = 0;
            }
            Self::Up if current_window == Windows::ThemeSelect => {
                let selected = {
                    let mutex = workspace.read().unwrap();
                    mutex.tree.selected
                };

                workspace.write().unwrap().tree.selected = selected.saturating_sub(1);
            }
            Self::Down if current_window == Windows::ThemeSelect => {
                let (mut selected, size) = {
                    let mutex = workspace.read().unwrap();
                    (mutex.tree.selected, mutex.config.themes.len())
                };

                if size != 0 && selected < size - 1 {
                    selected += 1;
                }
                workspace.write().unwrap().tree.selected = selected;
            }
            Self::Select if current_window == Windows::ThemeSelect => {
                let selected = {
                    let mutex = workspace.read().unwrap();
                    let mut sorted_names: Vec<&String> = mutex.config.themes.keys().collect();
                    sorted_names.sort();

                    let index = mutex.tree.selected;
                    sorted_names[index].to_string()
                };

                workspace.write().unwrap().config.selected_theme = selected;
            }

            // playlist-related functions
            Self::PlaylistSave if current_window == Windows::None => {
                workspace.write().unwrap().stdin_buffer.clear();
                workspace.write().unwrap().window = Windows::PlaylistSave;

                loop {
                    if event::poll(Duration::from_millis(100))? {
                        if let event::Event::Key(key_event) = event::read()? {
                            if let KeyEventKind::Release = key_event.kind {
                                continue;
                            }

                            if let KeyCode::Char(ch) = key_event.code {
                                workspace.write().unwrap().stdin_buffer.push(ch);
                                continue;
                            }

                            if let KeyCode::Esc = key_event.code {
                                let mut mutex = workspace.write().unwrap();
                                mutex.window = Windows::None;
                                mutex.stdin_buffer.clear();
                                return Ok(());
                            }

                            if let KeyCode::Backspace = key_event.code {
                                workspace.write().unwrap().stdin_buffer.pop();
                                continue;
                            }

                            if let KeyCode::Enter = key_event.code {
                                let (save_path, name) = {
                                    let mutex = workspace.read().unwrap();

                                    (
                                        mutex.config.playlists_folder.clone(),
                                        mutex.stdin_buffer.clone(),
                                    )
                                };

                                let queue = player.read().unwrap().queue.clone();
                                Saver::save_playlist(&save_path, name.clone(), queue)?;

                                let mut mutex = workspace.write().unwrap();
                                mutex.stdin_buffer.clear();
                                mutex.window = Windows::None;
                                mutex.tree.playlists.push(name);

                                return Ok(());
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
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
