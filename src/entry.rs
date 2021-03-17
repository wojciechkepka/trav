use anyhow::Result;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{fs, io};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
};

#[derive(Debug)]
pub struct DirEntry {
    inner: fs::DirEntry,
}

impl From<fs::DirEntry> for DirEntry {
    fn from(entry: fs::DirEntry) -> Self {
        DirEntry { inner: entry }
    }
}

impl DirEntry {
    pub fn metadata(&self) -> io::Result<fs::Metadata> {
        self.inner.metadata()
    }

    pub fn file_type(&self) -> io::Result<fs::FileType> {
        self.inner.file_type()
    }

    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }

    pub fn file_name(&self) -> OsString {
        self.inner.file_name()
    }

    pub fn as_list_item(&self) -> ListItem {
        let mut lines = vec![];

        if let Ok(metadata) = self.inner.metadata() {
            let file_type = metadata.file_type();

            let symbol = if file_type.is_dir() {
                "📁"
            } else if file_type.is_file() {
                "📄"
            } else {
                "🔗"
            };

            lines.push(Spans::from(format!(
                "{} {}",
                symbol,
                self.inner.file_name().to_string_lossy().to_string()
            )));

            lines.push(Spans::from(format!("{} B", metadata.len())));
        } else {
            lines.push(Spans::from(format!(
                "{}",
                self.inner.file_name().to_string_lossy().to_string()
            )));
        }

        ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
    }
}

pub fn get_ok_entries(path: &Path) -> Result<Vec<DirEntry>> {
    let mut entries = vec![];
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            entries.push(DirEntry::from(entry));
        }
    }

    Ok(entries)
}

pub fn styled_file_entries(title: String, entries: Vec<ListItem>) -> List {
    List::new(entries)
        .block(
            Block::default().borders(Borders::ALL).title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
        )
        .style(Style::default().bg(Color::Black))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("-> ")
}
