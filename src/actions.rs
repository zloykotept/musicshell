#![allow(dead_code, unused_variables)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Action {
    None,
    Up,
    Down,
    ChildDir,
    ParentDir,
    Exit,
    Copy,
    Delete,
    Paste,
    Cut,
    Rename,
    Play,
    CreateDir,
    Find,
    ToggleVisualMode,
    SelectAll,
    ToggleTreeView,
    ClearQeue,
    AddToQeue,
    AddAllToQeue,
    TogglePause,
    ToggleRepeat,
    QeueNext,
    QeuePrev,
    RewindForward(usize),
    RewindBack(usize),
    VolumeDecrease(usize),
    VolumeIncrease(usize),
    SelectTheme,
}

impl Action {
    pub fn from_str(action_str: &str) -> Option<Self> {
        match action_str {
            "Up" => Some(Action::Up),
            "Down" => Some(Action::Down),
            "ChildDir" => Some(Action::ChildDir),
            "ParentDir" => Some(Action::ParentDir),
            "Exit" => Some(Action::Exit),
            "Copy" => Some(Action::Copy),
            "Delete" => Some(Action::Delete),
            "Paste" => Some(Action::Paste),
            "Cut" => Some(Action::Cut),
            "Rename" => Some(Action::Rename),
            "Play" => Some(Action::Play),
            "CreateDir" => Some(Action::CreateDir),
            "Find" => Some(Action::Find),
            "ToggleVisualMode" => Some(Action::ToggleVisualMode),
            "SelectAll" => Some(Action::SelectAll),
            "ToggleTreeView" => Some(Action::ToggleTreeView),
            "ClearQeue" => Some(Action::ClearQeue),
            "AddToQeue" => Some(Action::AddToQeue),
            "AddAllToQeue" => Some(Action::AddAllToQeue),
            "TogglePause" => Some(Action::TogglePause),
            "ToggleRepeat" => Some(Action::ToggleRepeat),
            "QeueNext" => Some(Action::QeueNext),
            "QeuePrev" => Some(Action::QeuePrev),
            "SelectTheme" => Some(Action::SelectTheme),
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
}
