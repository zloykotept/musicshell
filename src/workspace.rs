#![allow(dead_code, unused_variables)]
use std::{cmp::Ordering, collections::VecDeque, env, path::PathBuf};

use anyhow::Result;

use crate::config::Config;

// Workspace section =======================================
pub struct Workspace {
    pub config: Config,
    pub tree: Tree,
}

impl Workspace {
    pub fn new(config: Config) -> Result<Self> {
        let tree = Tree::new()?;
        Ok(Workspace { config, tree })
    }
}

// Tree and All Tree related ============================
pub enum TreeState {
    Files,
    Qeue,
}

pub struct Tree {
    pub cwd: PathBuf,
    pub path_list: Vec<PathBuf>,
    pub qeue_list: VecDeque<PathBuf>,
    pub selected: (usize, usize),
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
        let qeue_list = VecDeque::new();

        Ok(Tree {
            cwd,
            path_list,
            qeue_list,
            selected: (0, 0),
            state: TreeState::Files,
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
