use std::{collections::HashMap, fs, path::PathBuf};

use crate::workspace::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Config {
    pub keymap_local: HashMap<KeyEvent, Action>,
    pub keymap_global: HashMap<KeyEvent, Action>,
}

impl Config {
    pub fn new(parser: &Parser) -> Self {
        let keymap_local = parser.parse_keys(false);
        let keymap_global = parser.parse_keys(true);
        Config {
            keymap_local,
            keymap_global,
        }
    }
}

pub struct Parser {
    std_config: String,
    act_config: String,
}

impl Parser {
    pub fn new(file: PathBuf) -> Self {
        let std_config = include_str!("../config_sample/config").to_string();
        let act_config = fs::read_to_string(&file).unwrap();
        Parser {
            std_config,
            act_config,
        }
    }

    pub fn parse_keys(&self, global: bool) -> HashMap<KeyEvent, Action> {
        let vars_act = if global {
            self.parse_vars("keymaps_global", false)
        } else {
            self.parse_vars("keymaps_local", false)
        };
        let vars_std = if global {
            self.parse_vars("keymaps_global", true)
        } else {
            self.parse_vars("keymaps_local", true)
        };

        let mut vars_clear: Vec<(&str, Action)> = vec![];
        for (var_name, var_value) in vars_std.into_iter() {
            let mut action: Action = Action::None;
            if let Some((_, new_value)) = vars_act.iter().find(|(x, _)| *x == var_name) {
                if new_value.contains(" args ") {
                    let splitted: Vec<&str> = new_value.split(" args ").collect();
                    if let Ok(arg) = splitted[1].parse::<usize>() {
                        if let Some(x) = Action::from_str_arg(splitted[0], arg) {
                            action = x;
                        } else {
                            continue;
                        }
                    }
                } else {
                    if let Some(x) = Action::from_str(*new_value) {
                        action = x;
                    } else {
                        continue;
                    }
                }

                vars_clear.push((var_name, action));
            } else {
                if var_value.contains(" args ") {
                    let splitted: Vec<&str> = var_value.split(" args ").collect();
                    if let Ok(arg) = splitted[1].parse::<usize>() {
                        if let Some(x) = Action::from_str_arg(splitted[0], arg) {
                            action = x;
                        } else {
                            continue;
                        }
                    }
                } else {
                    if let Some(x) = Action::from_str(var_value) {
                        action = x;
                    } else {
                        continue;
                    }
                }

                vars_clear.push((var_name, action));
            }
        }

        vars_clear
            .into_iter()
            .filter_map(|(keys_str, action)| {
                let keys_splitted: Vec<&str> = keys_str.split('+').collect();
                let mut key_modifiers = KeyModifiers::empty();
                let mut key_code = KeyCode::Null;

                for key in keys_splitted.into_iter() {
                    match key {
                        "SHIFT" => key_modifiers.insert(KeyModifiers::SHIFT),
                        "ALT" => key_modifiers.insert(KeyModifiers::ALT),
                        "CTRL" => key_modifiers.insert(KeyModifiers::CONTROL),
                        "SPACE" => key_code = KeyCode::Char(' '),
                        "ARROW_UP" => key_code = KeyCode::Up,
                        "ARROW_DOWN" => key_code = KeyCode::Down,
                        "ARROW_RIGHT" => key_code = KeyCode::Right,
                        "ARROW_LEFT" => key_code = KeyCode::Left,
                        "TAB" => key_code = KeyCode::Tab,
                        "BACKTAB" => key_code = KeyCode::BackTab,
                        "BACKSPACE" => key_code = KeyCode::Backspace,
                        "DEL" => key_code = KeyCode::Delete,
                        "ENTER" => key_code = KeyCode::Enter,
                        other => {
                            if other.len() == 1 {
                                key_code = KeyCode::Char(other.chars().last().unwrap())
                            } else {
                                return None;
                            }
                        }
                    }
                }

                let key_event = KeyEvent::new(key_code, key_modifiers);
                Some((key_event, action))
            })
            .collect()
    }

    fn parse_vars<'a>(&'a self, block: &str, std: bool) -> Vec<(&'a str, &'a str)> {
        let mut in_block = false;

        let iterator = if std {
            self.std_config.lines()
        } else {
            self.act_config.lines()
        };

        iterator
            .map(|line| {
                if let Some(i) = line.find('#') {
                    &line[..i]
                } else {
                    line
                }
            })
            .filter_map(|line| {
                let line = line.trim();

                if line == format!("[{}]", block) {
                    in_block = true;
                } else if line.starts_with('[') && line.ends_with(']') {
                    in_block = false;
                }

                if in_block {
                    let splitted: Vec<&str> = line.split(" = ").collect();
                    if splitted.len() == 2 {
                        Some((splitted[0], splitted[1]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn parse_subblocks<'a>(
        &'a self,
        block: &str,
        std: bool,
    ) -> Vec<(&'a str, Vec<(&'a str, &'a str)>)> {
        let mut in_block = false;
        let mut result = Vec::new();
        let mut current_subblock = "";
        let mut current_entries = Vec::new();

        let iterator = if std {
            self.std_config.lines()
        } else {
            self.act_config.lines()
        };

        let clean = iterator.map(|line| {
            if let Some(i) = line.find('#') {
                &line[..i]
            } else {
                line
            }
        });

        for line in clean {
            let line = line.trim();

            if line == format!("[{}]", block) {
                in_block = true;
                continue;
            } else if line.starts_with('[') && line.ends_with(']') {
                in_block = false;
                continue;
            }

            if in_block {
                if line.starts_with('(') && line.ends_with(')') {
                    if !current_subblock.is_empty() && !current_entries.is_empty() {
                        result.push((current_subblock, current_entries.clone()));
                    }
                    current_entries.clear();
                    current_subblock = &line[1..line.len() - 1];
                } else {
                    let splitted: Vec<&str> = line.split(" = ").collect();
                    if splitted.len() == 2 {
                        current_entries.push((splitted[0], splitted[1]));
                    }
                }
            }
        }

        if !current_subblock.is_empty() && !current_entries.is_empty() {
            result.push((current_subblock, current_entries.clone()));
        }
        result
    }
}

pub mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::config::Action;

    use super::Parser;

    const MOCK_CONFIG: &str = "
        # Some dick cum
        [bad_block]
        # shitty ass
        file = some some
        
        [block_norm]
        some shit isnt right = lkfjsdfsdlf = shit dick;
        j = action Up # comment
        k = action Down

        [themes]
        (1)
        text = some
        (2)
        text = some2";

    const MOCK_CONFIG_STD: &str = "
    [keymaps_local]
    SHIFT+SPACE = Exit
    CTRL+h = RewindBack args 5
    q = Exit
    j = Up
        ";

    const MOCK_CONFIG_ACT: &str = "
    [keymaps_local]
    j = Down
        ";

    #[test]
    fn parser_get_keymaps() {
        let parser_mock = Parser {
            std_config: String::from(MOCK_CONFIG_STD),
            act_config: String::from(MOCK_CONFIG_ACT),
        };

        let res = parser_mock.parse_keys(false);
        let mut res_to_check: HashMap<KeyEvent, Action> = HashMap::new();
        res_to_check.insert(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT),
            Action::Exit,
        );
        res_to_check.insert(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            Action::RewindBack(5),
        );
        res_to_check.insert(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
            Action::Exit,
        );
        res_to_check.insert(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
            Action::Down,
        );
        assert_eq!(res, res_to_check);
    }

    #[test]
    fn parser_vars() {
        let parser_mock = Parser {
            std_config: String::new(),
            act_config: String::from(MOCK_CONFIG),
        };

        let res = parser_mock.parse_vars("block_norm", false);
        assert_eq!(res, vec![("j", "action Up"), ("k", "action Down")]);
    }

    #[test]
    fn parser_subblocks() {
        let parser_mock = Parser {
            std_config: String::new(),
            act_config: String::from(MOCK_CONFIG),
        };

        let res = parser_mock.parse_subblocks("themes", false);
        assert_eq!(
            res,
            vec![
                ("1", vec![("text", "some")]),
                ("2", vec![("text", "some2")])
            ]
        );
    }
}
