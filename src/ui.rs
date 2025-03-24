use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::{anyhow, Result};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    DefaultTerminal,
};

use crate::{
    actions::MUSIC_EXTENSIONS,
    player::Player,
    workspace::{TreeState, Windows, Workspace, PLAYLIST_FILE_EXT},
};

pub struct UI;

impl UI {
    pub fn draw_cycle(
        workspace: Arc<RwLock<Workspace>>,
        player: Arc<RwLock<Player>>,
        mut terminal: DefaultTerminal,
    ) -> Result<()> {
        let config = {
            let ctx = workspace.read().unwrap();
            ctx.config.clone()
        };
        let mut selected_theme = config.selected_theme;
        let mut theme = config
            .themes
            .get(&selected_theme)
            .ok_or_else(|| anyhow!("Theme name in config invalid"))?
            .clone();
        let mut list_state = ListState::default();

        loop {
            let ctx = workspace.read().unwrap();
            if !ctx.running {
                break;
            }
            let player_mutex = player.read().unwrap();

            if ctx.config.selected_theme != selected_theme {
                selected_theme = ctx.config.selected_theme.clone();

                theme = ctx.config.themes.get(&selected_theme).unwrap().clone();
            }

            terminal.draw(|frame| {
                // layout main window ========================================
                let size = frame.area();
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                    .split(size);

                // style =======================================
                let error_style = Style::default()
                    .bg(Color::Rgb(theme.error[0], theme.error[1], theme.error[2]))
                    .fg(Color::Rgb(
                        theme.error_text[0],
                        theme.error_text[1],
                        theme.error_text[2],
                    ));
                let block_style = Style::default()
                    .bg(Color::Rgb(
                        theme.background[0],
                        theme.background[1],
                        theme.background[2],
                    ))
                    .fg(Color::Rgb(
                        theme.border[0],
                        theme.border[1],
                        theme.border[2],
                    ));
                let title_style = Style::default().fg(Color::Rgb(
                    theme.text_headline[0],
                    theme.text_headline[1],
                    theme.text_headline[2],
                ));
                let highlighted_style = Style::default()
                    .bg(Color::Rgb(
                        theme.highlighted[0],
                        theme.highlighted[1],
                        theme.highlighted[2],
                    ))
                    .fg(Color::Rgb(
                        theme.text_headline[0],
                        theme.text_headline[1],
                        theme.text_headline[2],
                    ));
                let text_style =
                    Style::default().fg(Color::Rgb(theme.text[0], theme.text[1], theme.text[2]));
                let directory_style = Style::default().fg(Color::Rgb(
                    theme.directories[0],
                    theme.directories[1],
                    theme.directories[2],
                ));
                let music_file_style =
                    Style::default().fg(Color::Rgb(theme.music[0], theme.music[1], theme.music[2]));
                let progress_style = Style::default().fg(Color::Rgb(
                    theme.progress_bar_elapsed[0],
                    theme.progress_bar_elapsed[1],
                    theme.progress_bar_elapsed[2],
                ));

                // player block ============================================
                let player_block = Block::default()
                    .title(ctx.tree.cwd.display().to_string())
                    .borders(Borders::ALL)
                    .style(block_style)
                    .title_style(title_style)
                    .padding(ratatui::widgets::Padding {
                        left: 1,
                        right: 1,
                        top: 1,
                        bottom: 0,
                    });

                // player block layout =====================================
                let player_block_area = player_block.inner(layout[1]);
                let layout_player = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Length(3)])
                    .split(player_block_area);

                // tree ===================================================
                let tree_state = Some(ctx.tree.state.clone())
                    .map(|x| {
                        if x == TreeState::Files {
                            "Files"
                        } else if x == TreeState::Queue {
                            "Queue"
                        } else {
                            "Playlists"
                        }
                    })
                    .unwrap();
                let tree_block = Block::default()
                    .title(tree_state)
                    .borders(Borders::ALL)
                    .style(block_style)
                    .title_style(title_style);

                // file metadata ===========================================
                let repeat_icon = if player_mutex.repeat {
                    '\u{f0458}'
                } else {
                    '\u{f0456}'
                };
                let metadata_str = format!(
                    "Now playing: {}\nVolume: {}%\nRepeat: {}\n",
                    player_mutex.now_playing.clone(),
                    player_mutex.get_volume_percantage(),
                    repeat_icon,
                );
                let song_metadata = Paragraph::new(metadata_str)
                    .style(text_style)
                    .alignment(Alignment::Center);

                // time formatting ===========================================
                let current_mins = player_mutex.get_duration_current() / 60;
                let current_secs = player_mutex.get_duration_current() - 60 * current_mins;
                let current_secs = if current_secs < 10 {
                    "0".to_string() + &current_secs.to_string()
                } else {
                    current_secs.to_string()
                };
                let full_mins = player_mutex.get_duration_total() / 60;
                let full_secs = player_mutex.get_duration_total() - 60 * full_mins;
                let full_secs = if full_secs < 10 {
                    "0".to_string() + &full_secs.to_string()
                } else {
                    full_secs.to_string()
                };

                // statusbar time and block ==============================================
                let statusbar_str = format!(
                    " {}:{} / {}:{} ",
                    current_mins, current_secs, full_mins, full_secs
                );

                let pause_icon = if player_mutex.is_paused() {
                    " \u{f03e4} "
                } else {
                    " \u{f040a} "
                };
                let statusbar = Block::default()
                    .title(statusbar_str)
                    .title_bottom(pause_icon)
                    .borders(Borders::ALL)
                    .style(block_style)
                    .title_style(title_style)
                    .title_alignment(Alignment::Center);

                // status bar poloska =============================================
                let percantage = if player_mutex.get_duration_total() != 0 {
                    (player_block_area.width as usize).saturating_sub(2) as f32
                        * (player_mutex.get_duration_current() as f32
                            / player_mutex.get_duration_total() as f32)
                } else {
                    0.0
                };
                let statusbar_progress = Paragraph::new("\u{2588}".repeat(percantage as usize))
                    .style(progress_style)
                    .block(statusbar);

                // list items ======================================================
                let list_items: Vec<ListItem>;
                if ctx.tree.state == TreeState::Files {
                    list_items = ctx
                        .tree
                        .path_list
                        .iter()
                        .map(|path| {
                            let path_str = path.file_name().unwrap().to_str().unwrap();
                            if path.is_dir() {
                                let formatted = format!("{} {}", '\u{ea83}', path_str);
                                return ListItem::new(formatted).style(directory_style);
                            } else if let Some(x) = path.extension() {
                                let x = x.to_str().unwrap();
                                if MUSIC_EXTENSIONS.contains(&x) {
                                    let formatted = format!("{} {}", '\u{f0387}', path_str);
                                    return ListItem::new(formatted).style(music_file_style);
                                } else if x == PLAYLIST_FILE_EXT {
                                    let formatted = format!("{} {}", '\u{f0cb8}', path_str);
                                    return ListItem::new(formatted).style(progress_style);
                                }
                            }
                            ListItem::new(path_str)
                        })
                        .collect();
                } else if ctx.tree.state == TreeState::Queue {
                    list_items = player_mutex
                        .queue
                        .iter()
                        .enumerate()
                        .map(|(index, path)| {
                            if !player_mutex.queue.is_empty()
                                && index == player_mutex.song_index.saturating_sub(1)
                            {
                                return ListItem::new(path.file_name().unwrap().to_str().unwrap())
                                    .style(music_file_style);
                            }
                            ListItem::new(path.file_name().unwrap().to_str().unwrap())
                        })
                        .collect();
                } else {
                    list_items = ctx
                        .tree
                        .playlists
                        .iter()
                        .map(|name| ListItem::new(name.clone()).style(progress_style))
                        .collect();
                }

                // list itself ===============================================
                let list = List::new(list_items)
                    .block(tree_block)
                    .highlight_style(highlighted_style)
                    .style(text_style);

                // theme select window block and layout
                let layout_themes_block_horizontal_split = Layout::default()
                    .direction(Direction::Horizontal)
                    .flex(Flex::Center)
                    .constraints([Constraint::Percentage(33)])
                    .split(size);
                let layout_themes_block = Layout::default()
                    .direction(Direction::Vertical)
                    .flex(Flex::Center)
                    .constraints([Constraint::Percentage(80)])
                    .split(layout_themes_block_horizontal_split[0]);
                let themes_block = Block::default()
                    .title("Select theme")
                    .title_style(title_style)
                    .style(block_style)
                    .borders(Borders::ALL);

                // theme select window list items
                let mut themes_list_items: Vec<ListItem> = vec![];
                if ctx.window == Windows::ThemeSelect {
                    let mut sorted_names: Vec<&String> = config.themes.keys().collect();
                    sorted_names.sort();
                    themes_list_items = sorted_names
                        .iter()
                        .map(|name| ListItem::new(name.to_string()))
                        .collect();
                }

                // theme select window list
                let themes_list = List::new(themes_list_items)
                    .block(themes_block)
                    .style(text_style)
                    .highlight_style(highlighted_style);

                // error popup
                let mut error_paragraph = Paragraph::default();
                let error_block = Block::default().style(error_style);

                if let Windows::Error(e_str) = &ctx.window {
                    error_paragraph = Paragraph::new(e_str.to_string())
                        .style(error_style)
                        .alignment(Alignment::Center);
                }
                let layout_error_horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .flex(Flex::Center)
                    .constraints([Constraint::Percentage(50)])
                    .split(size);
                let layout_error = Layout::default()
                    .direction(Direction::Vertical)
                    .flex(Flex::Center)
                    .constraints([Constraint::Percentage(50)])
                    .split(layout_error_horizontal[0]);

                // save playlist input
                let save_playlist_block = Block::default()
                    .title("Enter playlist name")
                    .title_style(title_style)
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .style(block_style);
                let layout_save_playlist_horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .flex(Flex::Center)
                    .constraints([Constraint::Length(32)])
                    .split(size);
                let layout_save_playlist = Layout::default()
                    .direction(Direction::Vertical)
                    .flex(Flex::Center)
                    .constraints([Constraint::Length(3)])
                    .split(layout_save_playlist_horizontal[0]);

                // save playlist input text
                let save_playlist_widget = Paragraph::new(ctx.stdin_buffer.clone() + "\u{258f}")
                    .block(save_playlist_block)
                    .style(text_style);

                // drawing what is needed
                list_state.select(Some(ctx.tree.selected));

                if ctx.window == Windows::None {
                    frame.render_stateful_widget(list, layout[0], &mut list_state);
                } else {
                    frame.render_stateful_widget(list, layout[0], &mut ListState::default());
                }
                frame.render_widget(player_block, layout[1]);
                frame.render_widget(song_metadata, layout_player[0]);
                frame.render_widget(statusbar_progress, layout_player[1]);

                if ctx.window == Windows::ThemeSelect {
                    frame.render_widget(Clear, layout_themes_block[0]);
                    frame.render_stateful_widget(
                        themes_list,
                        layout_themes_block[0],
                        &mut list_state,
                    );
                } else if let Windows::Error(_) = ctx.window {
                    frame.render_widget(Clear, layout_error[0]);
                    frame.render_widget(error_paragraph, layout_error[0]);
                } else if ctx.window == Windows::PlaylistSave {
                    frame.render_widget(Clear, layout_save_playlist[0]);
                    frame.render_widget(save_playlist_widget, layout_save_playlist[0]);
                }
            })?;

            drop(ctx);
            drop(player_mutex);
            thread::sleep(Duration::from_millis(32));
        }

        Ok(())
    }
}
