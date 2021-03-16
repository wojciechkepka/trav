pub mod list;

use anyhow::Result;
use std::{fs, io, path::Path};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{Backend, Term};

pub fn get_terminal() -> Result<Term> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn get_ok_entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut entries = vec![];
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            entries.push(entry);
        }
    }

    Ok(entries)
}

pub fn entry_as_list_item(entry: &fs::DirEntry) -> ListItem {
    let mut lines = vec![];

    if let Ok(metadata) = entry.metadata() {
        let file_type = metadata.file_type();
        let symbol = if file_type.is_dir() {
            "📁"
        } else if file_type.is_symlink() {
            "🔗"
        } else {
            "📄"
        };

        lines.push(Spans::from(format!(
            "{} {}",
            symbol,
            entry.file_name().to_string_lossy().to_string()
        )));

        lines.push(Spans::from(format!("{} B", metadata.len())));
    } else {
        lines.push(Spans::from(format!(
            "{}",
            entry.file_name().to_string_lossy().to_string()
        )));
    }

    ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
}

pub fn styled_file_entries(entries: Vec<ListItem>) -> List {
    List::new(entries)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .style(Style::default().bg(Color::Black))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("-> ")
}

pub fn err_paragraph<'err>(err: &'err str) -> Paragraph<'err> {
    Paragraph::new(err)
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::LightRed)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left)
}

pub fn layout(f: &mut Frame<Backend>, with_error: bool) -> Vec<Rect> {
    let constraints = if with_error {
        [Constraint::Min(3), Constraint::Min(3), Constraint::Min(90)].as_ref()
    } else {
        [Constraint::Min(3), Constraint::Percentage(98)].as_ref()
    };

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size())
}
