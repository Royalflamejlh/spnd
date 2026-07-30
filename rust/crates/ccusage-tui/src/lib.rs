//! Interactive terminal UI for Claude Code usage reports.
//!
//! Loads usage data through the Claude adapter before the terminal enters raw
//! mode, so loader progress renders normally, then hands the shaped rows to a
//! ratatui event loop with daily, monthly, and session tabs.
mod app;
mod data;
mod input;
mod ui;

use ccusage_core::{Context, Result, cli::SharedArgs};
use ratatui::crossterm::event::{self, Event, KeyEventKind};

pub fn run(shared: SharedArgs) -> Result<()> {
    let tables = data::load(&shared)?;
    if tables.is_empty() {
        println!("No Claude Code usage data found.");
        return Ok(());
    }
    let mut app = app::App::new(tables, shared.order);
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut app::App) -> Result<()> {
    while !app.should_quit {
        terminal
            .draw(|frame| ui::draw(frame, app))
            .context("failed to draw terminal frame")?;
        if let Event::Key(key) = event::read().context("failed to read terminal event")?
            && key.kind == KeyEventKind::Press
        {
            input::handle_key(app, key);
        }
    }
    Ok(())
}
