use anyhow::Result;
use std::path::Path;
use trav::{app::TravApp, util::get_terminal};

fn main() -> Result<()> {
    let mut terminal = get_terminal()?;
    let mut app = TravApp::new::<&Path>(None)?;

    loop {
        terminal.draw(|mut f| {
            app.draw_frame(&mut f);
        })?;

        app.handle_event()?;

        if app.exit {
            break;
        }
    }

    Ok(())
}
