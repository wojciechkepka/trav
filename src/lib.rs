pub mod app;
pub mod entry;
pub mod events;
pub mod util;

use std::io::Stdout;
use termion::{input::MouseTerminal, raw::RawTerminal, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;
type Term = Terminal<Backend>;
