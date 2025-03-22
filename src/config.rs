#![allow(dead_code, unused_variables)]
use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use crate::actions::*;
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;
use toml::Value;

#[derive(Clone)]
pub struct Config {
    pub keymap_local: HashMap<KeyEvent, Action>,
    pub themes: HashMap<String, Theme>,
    pub selected_theme: String,
    pub playlists_folder: PathBuf,
}

impl Config {
    pub fn new(parser: &Parser) -> Result<Self> {
        let keymap_local = parser.parse_keys()?;
        let selected_theme = parser.parse_selected_theme()?;
        let playlists_folder = parser.parse_playlists_folder()?;
        let themes = parser.parse_themes()?;
        Ok(Config {
            keymap_local,
            themes,
            selected_theme,
            playlists_folder,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    #[serde(default)]
    pub text: [u8; 3],
    pub text_headline: [u8; 3],
    pub background: [u8; 3],
    pub border: [u8; 3],
    pub music: [u8; 3],
    pub progress_bar_elapsed: [u8; 3],
    pub highlighted: [u8; 3],
    pub directories: [u8; 3],
    pub error: [u8; 3],
    pub error_text: [u8; 3],
}

pub struct Parser {
    config: Value,
}

#[derive(Debug, Deserialize)]
struct KeymapEntry {
    key: String,
    action: String,
    #[serde(default)]
    mods: Option<Vec<String>>,
    #[serde(default)]
    arg: Option<usize>,
}

impl Parser {
    pub fn new(file: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(file)?;
        let config = toml::de::from_str(&content)?;
        Ok(Parser { config })
    }

    pub fn parse_keys(&self) -> Result<HashMap<KeyEvent, Action>> {
        let std_error = "Config file isn't structured as expected";

        let keys_list = self.config.get("keymaps");
        let keys_list = keys_list.ok_or_else(|| anyhow!(std_error))?;

        let mut keymap_entries = vec![];
        for (_, vars) in keys_list.as_table().unwrap() {
            if let Some(keymap) = vars.get("keymap") {
                if let Some(keymap_array) = keymap.as_array() {
                    for entry in keymap_array {
                        let entry_struct: KeymapEntry =
                            toml::de::from_str(&entry.as_table().unwrap().to_string())?;
                        keymap_entries.push(entry_struct);
                    }
                }
            }
        }

        let result = keymap_entries
            .into_iter()
            .map(|entry| {
                let action;
                if let Some(arg) = entry.arg {
                    action = Action::from_str_arg(&entry.action, arg).unwrap_or(Action::None);
                } else {
                    action = Action::from_str(&entry.action).unwrap_or(Action::None);
                }

                let key_code = match entry.key.as_str() {
                    "SPACE" => KeyCode::Char(' '),
                    "BACKSPACE" => KeyCode::Backspace,
                    "TAB" => KeyCode::Tab,
                    "BACKTAB" => KeyCode::BackTab,
                    "DEL" => KeyCode::Delete,
                    "ENTER" => KeyCode::Enter,
                    "ESC" => KeyCode::Esc,
                    "ARROW_UP" => KeyCode::Up,
                    "ARROW_DOWN" => KeyCode::Down,
                    "ARROW_LEFT" => KeyCode::Left,
                    "ARROW_RIGHT" => KeyCode::Right,
                    k if k.starts_with('F') => k[1..]
                        .parse::<u8>()
                        .ok()
                        .filter(|&n| (1..=12).contains(&n))
                        .map(KeyCode::F)
                        .unwrap_or(KeyCode::Null),
                    k if k.len() == 1 => KeyCode::Char(k.chars().next().unwrap()),
                    _ => KeyCode::Null,
                };

                let mut key_modifiers = KeyModifiers::empty();
                if let Some(mods) = entry.mods {
                    for modificator in mods {
                        match modificator.as_str() {
                            "SHIFT" => key_modifiers.insert(KeyModifiers::SHIFT),
                            "SUPER" => key_modifiers.insert(KeyModifiers::SUPER),
                            "ALT" => key_modifiers.insert(KeyModifiers::ALT),
                            "CTRL" => key_modifiers.insert(KeyModifiers::CONTROL),
                            _ => {}
                        }
                    }
                }

                let key_event = KeyEvent::new(key_code, key_modifiers);
                (key_event, action)
            })
            .collect();

        Ok(result)
    }

    pub fn parse_themes(&self) -> Result<HashMap<String, Theme>> {
        let std_err = "Can not parse any theme from config file, please use sample config if you don't know what are you doing";
        let themes = self.config.get("themes").ok_or_else(|| anyhow!(std_err))?;
        let mut themes_map: HashMap<String, Theme> = HashMap::new();

        for (theme_name, theme_vars) in themes.as_table().unwrap() {
            let theme: Theme = toml::de::from_str(&theme_vars.as_table().unwrap().to_string())?;
            themes_map.insert(theme_name.to_string(), theme);
        }

        Ok(themes_map)
    }

    pub fn parse_selected_theme(&self) -> Result<String> {
        let table = self
            .config
            .get("preferences")
            .ok_or_else(|| anyhow!("Expected \"preferences\" table to be in config file"))?
            .as_table()
            .ok_or_else(|| anyhow!("Expected \"preferences\" to be a table"))?;
        let selected_theme = table
            .get("selected_theme")
            .ok_or_else(|| anyhow!("Expected \"selected_theme\" to be in preferences table"))?
            .as_str()
            .ok_or_else(|| anyhow!("Expected \"selected_theme\" to be a string"))?;

        Ok(selected_theme.to_string())
    }

    pub fn parse_playlists_folder(&self) -> Result<PathBuf> {
        let table = self
            .config
            .get("preferences")
            .ok_or_else(|| anyhow!("Expected \"preferences\" table to be in config file"))?
            .as_table()
            .ok_or_else(|| anyhow!("Expected \"preferences\" to be a table"))?;
        let selected_theme = table
            .get("playlists_folder")
            .ok_or_else(|| anyhow!("Expected \"playlists_folder\" to be in preferences table"))?
            .as_str()
            .ok_or_else(|| anyhow!("Expected \"playlists_folder\" to be a string"))?;

        PathBuf::from_str(selected_theme).map_err(|e| anyhow!(e))
    }
}
