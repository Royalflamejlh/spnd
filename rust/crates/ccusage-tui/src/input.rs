//! Translates key and mouse events into actions; the mapping depends on
//! whether the breakdown popup is open.
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::{
    action::Action,
    app::{App, Granularity},
    hit::HitMap,
};

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
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => {
            if app.detail.is_some() {
                Action::Back
            } else {
                Action::Quit
            }
        }
        KeyCode::Backspace => Action::Back,
        KeyCode::Char('[') => Action::StepModel(-1),
        KeyCode::Char(']') => Action::StepModel(1),
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Action::NextTab,
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => Action::PrevTab,
        KeyCode::Down | KeyCode::Char('j') => Action::MoveBy(1),
        KeyCode::Up | KeyCode::Char('k') => Action::MoveBy(-1),
        KeyCode::PageDown => Action::MoveBy(10),
        KeyCode::PageUp => Action::MoveBy(-10),
        KeyCode::Home | KeyCode::Char('g') => Action::SelectFirst,
        KeyCode::End | KeyCode::Char('G') => Action::SelectLast,
        KeyCode::Char('s') => Action::ToggleSort,
        KeyCode::Char('d') => Action::SetGranularity(Granularity::Daily),
        KeyCode::Char('w') => Action::SetGranularity(Granularity::Weekly),
        KeyCode::Char('m') => Action::SetGranularity(Granularity::Monthly),
        KeyCode::Enter | KeyCode::Char('b') => Action::OpenBreakdown,
        _ => return None,
    })
}

pub(crate) fn mouse_action(app: &App, hits: &HitMap, mouse: MouseEvent) -> Option<Action> {
    let (x, y) = (mouse.column, mouse.row);
    if hits.popup_active() {
        return match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) if hits.popup_contains(x, y) => {
                hits.popup_hit(x, y)
            }
            MouseEventKind::Down(_) => Some(Action::CloseBreakdown),
            _ => None,
        };
    }
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => hits.hit(x, y),
        MouseEventKind::Down(MouseButton::Right) => Some(Action::Back),
        MouseEventKind::ScrollDown => Some(Action::MoveBy(1)),
        MouseEventKind::ScrollUp => Some(Action::MoveBy(-1)),
        MouseEventKind::Moved => {
            let hovered = hits.row_at(x, y);
            (hovered != app.hovered_row).then_some(Action::HoverRow(hovered))
        }
        _ => None,
    }
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

    fn mouse(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn left_click_dispatches_the_hit_target() {
        let app = App::new(tables(), SortOrder::Asc);
        let mut hits = HitMap::default();
        hits.register(ratatui::layout::Rect::new(0, 0, 5, 1), Action::NextTab);
        assert_eq!(
            mouse_action(
                &app,
                &hits,
                mouse(MouseEventKind::Down(MouseButton::Left), 2, 0)
            ),
            Some(Action::NextTab)
        );
        assert_eq!(
            mouse_action(
                &app,
                &hits,
                mouse(MouseEventKind::Down(MouseButton::Left), 9, 9)
            ),
            None
        );
    }

    #[test]
    fn wheel_moves_the_selection() {
        let app = App::new(tables(), SortOrder::Asc);
        let hits = HitMap::default();
        assert_eq!(
            mouse_action(&app, &hits, mouse(MouseEventKind::ScrollDown, 0, 0)),
            Some(Action::MoveBy(1))
        );
        assert_eq!(
            mouse_action(&app, &hits, mouse(MouseEventKind::ScrollUp, 0, 0)),
            Some(Action::MoveBy(-1))
        );
    }

    #[test]
    fn hover_emits_only_on_change() {
        let mut app = App::new(tables(), SortOrder::Asc);
        let mut hits = HitMap::default();
        hits.register(ratatui::layout::Rect::new(0, 2, 20, 1), Action::ClickRow(0));
        assert_eq!(
            mouse_action(&app, &hits, mouse(MouseEventKind::Moved, 3, 2)),
            Some(Action::HoverRow(Some(0)))
        );
        app.hovered_row = Some(0);
        assert_eq!(
            mouse_action(&app, &hits, mouse(MouseEventKind::Moved, 4, 2)),
            None
        );
    }

    #[test]
    fn clicks_outside_the_popup_close_it() {
        let mut app = App::new(tables(), SortOrder::Asc);
        app.show_breakdown = true;
        let mut hits = HitMap::default();
        hits.register(ratatui::layout::Rect::new(0, 0, 5, 1), Action::NextTab);
        hits.set_popup(ratatui::layout::Rect::new(10, 10, 20, 5));
        assert_eq!(
            mouse_action(
                &app,
                &hits,
                mouse(MouseEventKind::Down(MouseButton::Left), 2, 0)
            ),
            Some(Action::CloseBreakdown)
        );
        assert_eq!(
            mouse_action(
                &app,
                &hits,
                mouse(MouseEventKind::Down(MouseButton::Left), 15, 12)
            ),
            None
        );
        assert_eq!(
            mouse_action(&app, &hits, mouse(MouseEventKind::ScrollDown, 0, 0)),
            None
        );
    }
}
