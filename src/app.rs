use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{env, fs};
use termion::event::Key;
use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::events::{Event, Events};
use crate::util::{
    self, entry_as_list_item, err_paragraph, get_ok_entries, layout, list::StatefulList,
};
use crate::Backend;

pub struct TravApp {
    pub current_dir: PathBuf,
    pub entries: StatefulList<fs::DirEntry>,
    pub events: Events,
    pub exit: bool,
    pub err: Option<String>,
}

impl TravApp {
    pub fn new<P: AsRef<Path>>(base_dir: Option<P>) -> Result<TravApp> {
        let path = if let Some(path) = base_dir {
            path.as_ref().to_path_buf()
        } else {
            env::current_dir()?
        };

        Ok(TravApp {
            events: Events::new(),
            entries: StatefulList::with_items(get_ok_entries(path.as_path())?),
            current_dir: path,
            exit: false,
            err: None,
        })
    }

    pub fn load_entries(&mut self, path: PathBuf) -> Result<()> {
        self.entries = StatefulList::with_items(get_ok_entries(path.as_path())?);
        self.current_dir = path;

        Ok(())
    }

    fn heading(&self) -> Paragraph {
        Paragraph::new(self.current_dir.to_string_lossy().to_string())
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(Color::LightBlue)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left)
    }

    fn restart_err(&mut self) {
        if self.err.is_some() {
            self.err = None;
        }
    }

    pub fn handle_event(&mut self) -> Result<()> {
        match self.events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    self.exit = true;
                }
                Key::Left => {
                    self.restart_err();
                    if let Some(parent) = self.current_dir.parent() {
                        let parent = parent.to_path_buf();
                        self.load_entries(parent)?;
                    }
                }
                Key::Down => {
                    self.restart_err();
                    self.entries.next();
                }
                Key::Up => {
                    self.restart_err();
                    self.entries.previous();
                }
                Key::Right => {
                    self.restart_err();
                    if let Some(entry) = self.entries.current() {
                        match entry.metadata() {
                            Ok(md) => {
                                if md.is_dir() {
                                    let path = entry.path();
                                    self.load_entries(path)?;
                                } else {
                                    self.err = Some("entry is not a directory".to_string());
                                }
                            }
                            Err(e) => self.err = Some(e.to_string()),
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
        Ok(())
    }

    pub fn draw_frame(&mut self, mut f: &mut Frame<Backend>) {
        let entries: Vec<_> = self.entries.items.iter().map(entry_as_list_item).collect();
        let (error, is_err, file_idx) = if let Some(error) = &self.err {
            (error.to_string(), true, 2)
        } else {
            ("".to_string(), false, 1)
        };

        let chunks = layout(&mut f, is_err);

        f.render_widget(self.heading(), chunks[0]);

        if is_err {
            let err = err_paragraph(&error);
            f.render_widget(err, chunks[1]);
        }

        f.render_stateful_widget(
            util::styled_file_entries(entries),
            chunks[file_idx],
            &mut self.entries.state,
        );
    }
}
