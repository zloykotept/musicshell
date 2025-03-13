use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crossterm::event::KeyEvent;

pub enum Action {
    Up,
    Down,
    ChildDir,
    ParentDir,
    Exit,
}

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
        let mut maps = HashMap::new();
        let vars = if global {
            self.parse_vars("keymaps_global", false)
        } else {
            self.parse_vars("keymaps_local", false)
        };

        maps
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
    use std::path::PathBuf;

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
