use std::{collections::HashMap, fs, path::PathBuf};

use crate::workspace::*;
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;
use toml::Value;

pub struct Config {
    pub keymap_local: HashMap<KeyEvent, Action>,
    pub keymap_global: HashMap<KeyEvent, Action>,
}

impl Config {
    pub fn new(parser: &Parser) -> Result<Self> {
        let keymap_local = parser.parse_keys(false)?;
        let keymap_global = parser.parse_keys(true)?;
        Ok(Config {
            keymap_local,
            keymap_global,
        })
    }
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

                let mut key_code = KeyCode::Null;
                let mut key_modifiers = KeyModifiers::empty();
                match entry.key.as_str() {
                    "SPACE" => key_code = KeyCode::Char(' '),
                    "BACKSPACE" => key_code = KeyCode::Backspace,
                    "TAB" => key_code = KeyCode::Tab,
                    "BACKTAB" => key_code = KeyCode::BackTab,
                    "DEL" => key_code = KeyCode::Delete,
                    "ENTER" => key_code = KeyCode::Enter,
                    "ESC" => key_code = KeyCode::Esc,
                    "ARROW_UP" => key_code = KeyCode::Up,
                    "ARROW_DOWN" => key_code = KeyCode::Down,
                    "ARROW_LEFT" => key_code = KeyCode::Left,
                    "ARROW_RIGHT" => key_code = KeyCode::Right,
                    char => {
                        if char.len() == 1 {
                            key_code = KeyCode::Char(char.parse::<char>().unwrap());
                        }
                    }
                }
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
}
