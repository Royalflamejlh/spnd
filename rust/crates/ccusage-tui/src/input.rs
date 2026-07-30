//! Key handling for the report browser and the breakdown popup.
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub(crate) fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }
    if app.show_breakdown {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q' | 'b')
        ) {
            app.show_breakdown = false;
        }
        return;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
        KeyCode::Down | KeyCode::Char('j') => app.move_by(1),
        KeyCode::Up | KeyCode::Char('k') => app.move_by(-1),
        KeyCode::PageDown => app.move_by(10),
        KeyCode::PageUp => app.move_by(-10),
        KeyCode::Home | KeyCode::Char('g') => app.select_first(),
        KeyCode::End | KeyCode::Char('G') => app.select_last(),
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Enter | KeyCode::Char('b') => {
            if app.selected().is_some() {
                app.show_breakdown = true;
            }
        }
        _ => {}
    }
}
