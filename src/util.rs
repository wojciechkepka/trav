pub mod list;

use anyhow::Result;
use chrono::{offset::Utc, DateTime, TimeZone};
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use crate::Term;

const KILO: f64 = 1000.;
const MEGA: f64 = KILO * KILO;
const GIGA: f64 = KILO * KILO * KILO;
const TERA: f64 = KILO * KILO * KILO * KILO;

pub fn get_terminal() -> Result<Term> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Utc> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    Utc.timestamp(sec, nsec)
}

fn conv_metric(value: f64, unit: &str) -> String {
    let (val, u) = if value < KILO {
        (value, "")
    } else if KILO <= value && value < MEGA {
        (value / KILO, "K")
    } else if MEGA <= value && value < GIGA {
        (value / MEGA, "M")
    } else if GIGA <= value && value < TERA {
        (value / GIGA, "G")
    } else {
        (value / TERA, "T")
    };

    format!("{:.2}{}{}", val, u, unit)
}

pub fn conv_fb(bytes: f64) -> String {
    conv_metric(bytes, "B")
}

pub fn conv_b(bytes: u64) -> String {
    conv_fb(bytes as f64)
}
