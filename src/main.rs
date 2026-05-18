mod app;
mod build;
mod console;
mod editor;
mod events;
mod menu;
mod preview;
mod prompt;
mod settings;
mod status;

use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;

use crate::app::App;

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = App::new().and_then(|mut app| app.run(&mut terminal));
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}
