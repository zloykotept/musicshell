use std::{collections::HashMap, fs, path::PathBuf};

use crate::workspace::*;
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;
use toml::Value;

pub struct Config {
    pub keymap_local: HashMap<KeyEvent, Action>,
    pub keymap_global: HashMap<KeyEvent, Action>,
    pub themes: Vec<Theme>,
}

impl Config {
    pub fn new(parser: &Parser) -> Result<Self> {
        let keymap_local = parser.parse_keys(false)?;
        let keymap_global = parser.parse_keys(true)?;
        let themes = parser.parse_themes()?;
        Ok(Config {
            keymap_local,
            keymap_global,
            themes,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    #[serde(default)]
    name: String,
    text: String,
    text_headline: String,
    background: String,
    border: String,
    button: String,
    progress_bar_elapsed: String,
    progress_bar_remaining: String,
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

    pub fn parse_keys(&self, global: bool) -> Result<HashMap<KeyEvent, Action>> {
        let std_error = "Config file isn't structured as expected";

        let keys_list = if global {
            self.config.get("keymaps_global")
        } else {
            self.config.get("keymaps_local")
        };
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

    pub fn parse_themes(&self) -> Result<Vec<Theme>> {
        let std_err = "Can not parse any theme from config file, please use sample config if you don't know what are you doing";
        let themes = self.config.get("themes").ok_or_else(|| anyhow!(std_err))?;
        let mut themes_vec: Vec<Theme> = vec![];

        for (theme_name, theme_vars) in themes.as_table().unwrap() {
            let mut theme: Theme = toml::de::from_str(&theme_vars.as_table().unwrap().to_string())?;
            theme.name = theme_name.to_string();
            themes_vec.push(theme);
        }

        Ok(themes_vec)
    }
}
