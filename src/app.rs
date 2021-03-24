use anyhow::Result;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::{env, fs, io, process};
use termion::event::Key;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, ListState, Paragraph},
    Frame,
};

use crate::entry::{get_ok_entries, styled_file_entries, DirEntry};
use crate::events::{Event, Events};
use crate::util::list::StatefulList;
use crate::Backend;

#[derive(Debug)]
pub struct TravApp {
    pub cwd_path: PathBuf,
    pub cwd_entries: StatefulList<DirEntry>,
    pub cwd_idx: Option<usize>,
    pub parent: Option<(PathBuf, Vec<DirEntry>)>,
    pub parent_idx: Option<usize>,
    pub child_entries: Option<Vec<DirEntry>>,
    pub content: Option<String>,
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

        let mut app = TravApp {
            cwd_path: path.clone(),
            cwd_entries: StatefulList::new(),
            cwd_idx: None,
            parent: None,
            parent_idx: None,
            child_entries: None,
            content: None,
            events: Events::new(),
            exit: false,
            err: None,
        };
        app.load_entries(path, Some(1))?;
        app.handle_current_entry()?;

        Ok(app)
    }

    pub fn load_entries(&mut self, path: PathBuf, idx: Option<usize>) -> Result<()> {
        self.cwd_entries = StatefulList::with_items(get_ok_entries(path.as_path())?);
        if let Some(parent) = path.parent() {
            self.parent = Some((parent.to_path_buf(), get_ok_entries(parent)?));
        }
        self.cwd_path = path;

        self.cwd_entries.select(idx);
        self.cwd_idx = self.cwd_entries.current_idx();

        Ok(())
    }

    fn next_entry(&mut self) {
        self.cwd_idx = self.cwd_entries.next();
    }

    fn prev_entry(&mut self) {
        self.cwd_idx = self.cwd_entries.previous();
    }

    fn restart_err(&mut self) {
        if self.err.is_some() {
            self.err = None;
        }
    }

    fn handle_current_entry(&mut self) -> Result<()> {
        if let Some(entry) = self.cwd_entries.current() {
            match entry.metadata() {
                Ok(ref md) => {
                    let file_type = md.file_type();
                    if file_type.is_dir() {
                        self.child_entries = Some(get_ok_entries(entry.path().as_path())?);
                        return Ok(());
                    } else if file_type.is_symlink() {
                        if let Ok(entries) = get_ok_entries(entry.path().as_path()) {
                            self.child_entries = Some(entries);
                        }
                    } else if file_type.is_file() {
                        if let Ok(file) = fs::File::open(entry.path().as_path()) {
                            let reader = io::BufReader::new(file);
                            let mut lines = String::new();
                            for line in reader.lines().take(128) {
                                if let Ok(line) = line {
                                    lines.push_str(&line);
                                    lines.push('\n');
                                }
                            }

                            self.content = Some(lines);
                            self.child_entries = None;
                            self.err = None;
                        }
                    }
                }
                Err(e) => self.err = Some(e.to_string()),
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self) -> Result<()> {
        match self.events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    self.exit = true;
                }
                Key::Left => {
                    self.restart_err();
                    if let Some(parent) = self.cwd_path.parent() {
                        let parent = parent.to_path_buf();
                        self.load_entries(parent, self.parent_idx)?;
                    }
                    self.handle_current_entry()?;
                    self.parent_idx = None;
                }
                Key::Down => {
                    self.restart_err();
                    self.next_entry();
                    self.handle_current_entry()?;
                }
                Key::Up => {
                    self.restart_err();
                    self.prev_entry();
                    self.handle_current_entry()?;
                }
                Key::Right | Key::Char('\n') => {
                    self.restart_err();
                    if let Some(entry) = self.cwd_entries.current() {
                        if let Ok(md) = entry.metadata() {
                            let path = entry.path();
                            let file_type = md.file_type();
                            if file_type.is_dir() {
                                let idx = self.cwd_idx;
                                self.load_entries(path, Some(0))?;
                                self.handle_current_entry()?;
                                self.parent_idx = idx;
                                return Ok(());
                            } else if file_type.is_symlink() {
                                let idx = self.cwd_idx;
                                if let Ok(_) = self.load_entries(path, Some(0)) {
                                    self.handle_current_entry()?;
                                    self.parent_idx = idx;
                                    return Ok(());
                                }
                            } else if file_type.is_file() {
                                if let Err(e) = process::Command::new("xdg-open")
                                    .args(&[entry.path().to_string_lossy().to_string()])
                                    .spawn()
                                {
                                    self.err = Some(e.to_string());
                                }
                            }
                        }
                    }
                    self.handle_current_entry()?;
                }
                _ => {}
            },
            Event::Tick => {}
        }
        Ok(())
    }

    fn render_main_view(&mut self, mut f: &mut Frame<Backend>, rect: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(rect);

        if let Some((path, entries)) = &self.parent {
            render_entries(
                entries.iter(),
                path.to_string_lossy().to_string(),
                &mut f,
                chunks[0],
            );
        }

        render_stateful_entries(
            self.cwd_entries.items.iter(),
            self.cwd_path.to_string_lossy().to_string(),
            &mut self.cwd_entries.state,
            &mut f,
            chunks[1],
        );

        if let Some(current) = self.cwd_entries.current() {
            self.render_entry_info(current, &mut f, chunks[2]);
        }
    }

    fn render_entry_info(&self, entry: &DirEntry, mut frame: &mut Frame<Backend>, rect: Rect) {
        let _path = entry.path();
        let name = _path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| _path.to_string_lossy().to_string());

        if let Some(child_entries) = &self.child_entries {
            render_entries(child_entries.iter(), name, &mut frame, rect);
        } else {
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                name,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = if let Some(content) = &self.content {
                Paragraph::new(content.as_str()).block(block)
            } else {
                Paragraph::new("...").block(block)
            };

            frame.render_widget(paragraph, rect);
        }
    }

    pub fn draw_frame(&mut self, mut f: &mut Frame<Backend>) {
        let error = &self.err;
        let mut idx = 0;

        let chunks = main_layout(&mut f, error.is_some());

        if let Some(error) = error {
            render_error_msg(&error, &mut f, chunks[idx]);
            idx += 1;
        }

        //self.render_dbg(&mut f, chunks[idx]);
        //idx += 1;

        self.render_main_view(&mut f, chunks[idx]);
    }

    #[allow(dead_code)]
    fn render_dbg(&self, frame: &mut Frame<Backend>, rect: Rect) {
        let dbg = Paragraph::new(Spans::from(vec![
            Span::raw(format!("cwd_idx: {:?} ", self.cwd_idx)),
            Span::raw(format!("parent_idx: {:?}", self.parent_idx)),
        ]))
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::LightRed)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);

        frame.render_widget(dbg, rect);
    }
}

fn render_error_msg<S>(error: S, frame: &mut Frame<Backend>, rect: Rect)
where
    S: AsRef<str>,
{
    let err = Paragraph::new(error.as_ref())
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::LightRed)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);

    frame.render_widget(err, rect);
}

fn render_entries<'entry, I>(entries: I, title: String, frame: &mut Frame<Backend>, rect: Rect)
where
    I: Iterator<Item = &'entry DirEntry>,
{
    let entries: Vec<_> = entries.map(DirEntry::as_list_item).collect();
    frame.render_widget(styled_file_entries(title, entries), rect);
}

fn render_stateful_entries<'entry, I>(
    entries: I,
    title: String,
    mut state: &mut ListState,
    frame: &mut Frame<Backend>,
    rect: Rect,
) where
    I: Iterator<Item = &'entry DirEntry>,
{
    let entries: Vec<_> = entries.map(DirEntry::as_list_item).collect();
    frame.render_stateful_widget(styled_file_entries(title, entries), rect, &mut state);
}

pub fn main_layout(f: &mut Frame<Backend>, with_error: bool) -> Vec<Rect> {
    let constraints = if with_error {
        [Constraint::Min(3), Constraint::Percentage(90)].as_ref()
    } else {
        [Constraint::Percentage(97)].as_ref()
    };

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size())
}
