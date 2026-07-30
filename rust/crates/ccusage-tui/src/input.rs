//! Translates key events into actions; the mapping depends on whether the
//! breakdown popup is open.
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{action::Action, app::App};

pub(crate) fn key_action(app: &App, key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }
    if app.show_breakdown {
        return matches!(
            key.code,
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q' | 'b')
        )
        .then_some(Action::CloseBreakdown);
    }
    Some(match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Action::NextTab,
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => Action::PrevTab,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveBy(1),
        KeyCode::Up | KeyCode::Char('k') => Action::MoveBy(-1),
        KeyCode::PageDown => Action::MoveBy(10),
        KeyCode::PageUp => Action::MoveBy(-10),
        KeyCode::Home | KeyCode::Char('g') => Action::SelectFirst,
        KeyCode::End | KeyCode::Char('G') => Action::SelectLast,
        KeyCode::Char('s') => Action::ToggleSort,
        KeyCode::Enter | KeyCode::Char('b') => Action::OpenBreakdown,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use ccusage_core::cli::SortOrder;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::data::fixtures::tables;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn ctrl_c_quits_even_with_the_popup_open() {
        let mut app = App::new(tables(), SortOrder::Asc);
        app.show_breakdown = true;
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(key_action(&app, ctrl_c), Some(Action::Quit));
    }

    #[test]
    fn popup_swallows_navigation_keys() {
        let mut app = App::new(tables(), SortOrder::Asc);
        app.show_breakdown = true;
        assert_eq!(key_action(&app, key(KeyCode::Down)), None);
        assert_eq!(
            key_action(&app, key(KeyCode::Esc)),
            Some(Action::CloseBreakdown)
        );
    }

    #[test]
    fn browser_keys_map_to_actions() {
        let app = App::new(tables(), SortOrder::Asc);
        assert_eq!(
            key_action(&app, key(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        assert_eq!(key_action(&app, key(KeyCode::Tab)), Some(Action::NextTab));
        assert_eq!(
            key_action(&app, key(KeyCode::Char('j'))),
            Some(Action::MoveBy(1))
        );
        assert_eq!(
            key_action(&app, key(KeyCode::Char('s'))),
            Some(Action::ToggleSort)
        );
        assert_eq!(
            key_action(&app, key(KeyCode::Enter)),
            Some(Action::OpenBreakdown)
        );
        assert_eq!(key_action(&app, key(KeyCode::Char('x'))), None);
    }
}
