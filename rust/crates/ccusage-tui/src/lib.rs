//! Interactive terminal UI for Claude Code usage reports.
//!
//! Loads usage data through the Claude adapter before the terminal enters raw
//! mode, so loader progress renders normally, then hands the shaped rows to a
//! ratatui event loop with daily, monthly, and session tabs.
mod action;
mod app;
mod data;
mod hit;
mod input;
mod ui;

use ccusage_core::{Context, Result, cli::SharedArgs};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
};

pub fn run(shared: SharedArgs) -> Result<()> {
    let tables = data::load(&shared)?;
    if tables.is_empty() {
        println!("No Claude Code usage data found.");
        return Ok(());
    }
    let mut app = app::App::new(tables, shared.order);
    let mut terminal = ratatui::init();
    enable_mouse();
    let result = event_loop(&mut terminal, &mut app);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// Turns on mouse reporting and chains a panic hook that turns it off again
/// before ratatui's own hook restores the terminal.
fn enable_mouse() {
    if execute!(std::io::stdout(), EnableMouseCapture).is_err() {
        return;
    }
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
        hook(info);
    }));
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut app::App) -> Result<()> {
    let mut hits = hit::HitMap::default();
    while !app.should_quit {
        terminal
            .draw(|frame| ui::draw(frame, app, &mut hits))
            .context("failed to draw terminal frame")?;
        let action = match event::read().context("failed to read terminal event")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => input::key_action(app, key),
            Event::Mouse(mouse) => input::mouse_action(app, &hits, mouse),
            _ => None,
        };
        if let Some(action) = action {
            action::update(app, action);
        }
    }
    Ok(())
}
