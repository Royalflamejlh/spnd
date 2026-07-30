//! The single action vocabulary every input source maps into, and the update
//! function that applies one action to the application state.
use crate::app::{App, Granularity, Tab};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Quit,
    NextTab,
    PrevTab,
    SwitchTab(Tab),
    SetGranularity(Granularity),
    MoveBy(isize),
    SelectFirst,
    SelectLast,
    /// Mouse click on a table row: selects it, or opens the breakdown when
    /// the row is already selected.
    ClickRow(usize),
    HoverRow(Option<usize>),
    ToggleSort,
    OpenBreakdown,
    CloseBreakdown,
}

pub(crate) fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,
        Action::NextTab => app.next_tab(),
        Action::PrevTab => app.prev_tab(),
        Action::SwitchTab(tab) => app.switch_tab(tab),
        Action::SetGranularity(granularity) => app.set_granularity(granularity),
        Action::MoveBy(delta) => app.move_by(delta),
        Action::SelectFirst => app.select_first(),
        Action::SelectLast => app.select_last(),
        Action::ClickRow(index) => {
            if app.state_selected() == Some(index) {
                app.show_breakdown = true;
            } else {
                app.select_row(index);
            }
        }
        Action::HoverRow(row) => app.hovered_row = row,
        Action::ToggleSort => app.toggle_sort(),
        Action::OpenBreakdown => {
            if app.selected().is_some() {
                app.show_breakdown = true;
            }
        }
        Action::CloseBreakdown => app.show_breakdown = false,
    }
}

#[cfg(test)]
mod tests {
    use ccusage_core::cli::SortOrder;

    use super::*;
    use crate::{app::Tab, data::fixtures::tables};

    fn app() -> App {
        App::new(tables(), SortOrder::Asc)
    }

    #[test]
    fn quit_sets_should_quit() {
        let mut app = app();
        update(&mut app, Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn tab_cycling_wraps_both_directions() {
        let mut app = app();
        assert_eq!(app.tab, Tab::Usage);
        update(&mut app, Action::NextTab);
        assert_eq!(app.tab, Tab::Sessions);
        update(&mut app, Action::NextTab);
        assert_eq!(app.tab, Tab::Usage);
        update(&mut app, Action::PrevTab);
        assert_eq!(app.tab, Tab::Sessions);
    }

    #[test]
    fn granularity_switches_the_usage_rows() {
        let mut app = app();
        assert_eq!(app.rows().len(), 3);
        update(&mut app, Action::SetGranularity(Granularity::Weekly));
        assert_eq!(app.rows().len(), 1);
        assert_eq!(app.rows()[0].week.as_deref(), Some("2026-06-28"));
        update(&mut app, Action::SetGranularity(Granularity::Monthly));
        assert_eq!(app.rows()[0].month.as_deref(), Some("2026-07"));
    }

    #[test]
    fn granularity_shortcut_jumps_back_to_the_usage_tab() {
        let mut app = app();
        update(&mut app, Action::SwitchTab(Tab::Sessions));
        update(&mut app, Action::SetGranularity(Granularity::Monthly));
        assert_eq!(app.tab, Tab::Usage);
        assert_eq!(app.granularity, Granularity::Monthly);
    }

    #[test]
    fn each_view_keeps_its_own_selection() {
        let mut app = app();
        update(&mut app, Action::MoveBy(2));
        assert_eq!(app.state_selected(), Some(2));
        update(&mut app, Action::SetGranularity(Granularity::Weekly));
        assert_eq!(app.state_selected(), Some(0));
        update(&mut app, Action::SetGranularity(Granularity::Daily));
        assert_eq!(app.state_selected(), Some(2));
    }

    #[test]
    fn movement_clamps_to_table_bounds() {
        let mut app = app();
        update(&mut app, Action::MoveBy(-5));
        assert_eq!(app.states[0].selected(), Some(0));
        update(&mut app, Action::MoveBy(100));
        assert_eq!(app.states[0].selected(), Some(app.rows().len() - 1));
        update(&mut app, Action::SelectFirst);
        assert_eq!(app.states[0].selected(), Some(0));
        update(&mut app, Action::SelectLast);
        assert_eq!(app.states[0].selected(), Some(app.rows().len() - 1));
    }

    #[test]
    fn toggle_sort_reverses_rows_and_direction() {
        let mut app = app();
        let first_before = app.rows().first().unwrap().date.clone();
        let descending_before = app.descending();
        update(&mut app, Action::ToggleSort);
        assert_eq!(app.rows().last().unwrap().date, first_before);
        assert_ne!(app.descending(), descending_before);
        assert_eq!(app.states[0].selected(), Some(0));
    }

    #[test]
    fn switch_tab_jumps_directly_and_clears_hover() {
        let mut app = app();
        app.hovered_row = Some(1);
        update(&mut app, Action::SwitchTab(Tab::Sessions));
        assert_eq!(app.tab, Tab::Sessions);
        assert_eq!(app.hovered_row, None);
    }

    #[test]
    fn click_row_selects_then_opens_breakdown() {
        let mut app = app();
        update(&mut app, Action::ClickRow(1));
        assert_eq!(app.states[0].selected(), Some(1));
        assert!(!app.show_breakdown);
        update(&mut app, Action::ClickRow(1));
        assert!(app.show_breakdown);
    }

    #[test]
    fn click_row_ignores_out_of_range_indexes() {
        let mut app = app();
        update(&mut app, Action::ClickRow(999));
        assert_eq!(app.states[0].selected(), Some(0));
        assert!(!app.show_breakdown);
    }

    #[test]
    fn hover_row_updates_transient_state() {
        let mut app = app();
        update(&mut app, Action::HoverRow(Some(2)));
        assert_eq!(app.hovered_row, Some(2));
        update(&mut app, Action::HoverRow(None));
        assert_eq!(app.hovered_row, None);
    }

    #[test]
    fn breakdown_opens_only_with_a_selection() {
        let mut app = app();
        update(&mut app, Action::OpenBreakdown);
        assert!(app.show_breakdown);
        update(&mut app, Action::CloseBreakdown);
        assert!(!app.show_breakdown);

        let mut empty = App::new(crate::data::fixtures::empty_tables(), SortOrder::Asc);
        update(&mut empty, Action::OpenBreakdown);
        assert!(!empty.show_breakdown);
    }
}
