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
    /// Mouse click on a table row: selects it, or drills in when the row is
    /// already selected.
    ClickRow(usize),
    HoverRow(Option<usize>),
    ToggleSort,
    /// Opens the drill-down for `tables.models[index]`.
    OpenModel(usize),
    /// Steps the open model drill-down forward or backward through the model
    /// list, wrapping around.
    StepModel(isize),
    /// Closes the topmost layer: the breakdown popup, or the model
    /// drill-down.
    Back,
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
                drill_in(app);
            } else {
                app.select_row(index);
            }
        }
        Action::HoverRow(row) => app.hovered_row = row,
        Action::ToggleSort => app.toggle_sort(),
        Action::OpenModel(index) => app.open_model(index),
        Action::StepModel(delta) => app.step_model(delta),
        Action::Back => {
            if app.show_breakdown {
                app.show_breakdown = false;
            } else {
                app.close_detail();
            }
        }
        Action::OpenBreakdown => drill_in(app),
        Action::CloseBreakdown => app.show_breakdown = false,
    }
}

/// Enter on the models table opens the drill-down; everywhere else it opens
/// the selected row's breakdown popup.
fn drill_in(app: &mut App) {
    if app.tab == Tab::Models && app.detail.is_none() {
        if let Some(index) = app.state_selected() {
            app.open_model(index);
        }
    } else if app.selected().is_some() {
        app.show_breakdown = true;
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
    fn prev_tab_wraps_backwards() {
        let mut app = app();
        assert_eq!(app.tab, Tab::Usage);
        update(&mut app, Action::PrevTab);
        assert_eq!(app.tab, Tab::Models);
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
    fn tab_cycling_covers_all_three_tabs() {
        let mut app = app();
        update(&mut app, Action::NextTab);
        assert_eq!(app.tab, Tab::Sessions);
        update(&mut app, Action::NextTab);
        assert_eq!(app.tab, Tab::Models);
        update(&mut app, Action::NextTab);
        assert_eq!(app.tab, Tab::Usage);
    }

    #[test]
    fn open_model_builds_the_drill_down() {
        let mut app = app();
        update(&mut app, Action::OpenModel(0));
        let detail = app.detail.as_ref().unwrap();
        assert_eq!(detail.model, "claude-sonnet-5");
        assert_eq!(detail.rows.len(), 2);
        assert_eq!(app.tab, Tab::Models);
        assert_eq!(app.rows().len(), 2);
        update(&mut app, Action::Back);
        assert!(app.detail.is_none());
    }

    #[test]
    fn step_model_wraps_through_the_model_list() {
        let mut app = app();
        update(&mut app, Action::OpenModel(0));
        update(&mut app, Action::StepModel(1));
        assert_eq!(app.detail.as_ref().unwrap().model, "claude-fable-5");
        update(&mut app, Action::StepModel(1));
        assert_eq!(app.detail.as_ref().unwrap().model, "claude-sonnet-5");
        update(&mut app, Action::StepModel(-1));
        assert_eq!(app.detail.as_ref().unwrap().model, "claude-fable-5");
    }

    #[test]
    fn granularity_rebuckets_an_open_drill_down_in_place() {
        let mut app = app();
        update(&mut app, Action::OpenModel(0));
        update(&mut app, Action::SetGranularity(Granularity::Monthly));
        let detail = app.detail.as_ref().unwrap();
        assert_eq!(detail.rows.len(), 1);
        assert_eq!(detail.rows[0].month.as_deref(), Some("2026-07"));
        assert_eq!(app.tab, Tab::Models);
    }

    #[test]
    fn enter_on_the_models_tab_opens_the_drill_down() {
        let mut app = app();
        update(&mut app, Action::SwitchTab(Tab::Models));
        update(&mut app, Action::OpenBreakdown);
        assert!(app.detail.is_some());
        assert!(!app.show_breakdown);
    }

    #[test]
    fn click_on_selected_models_row_drills_in() {
        let mut app = app();
        update(&mut app, Action::SwitchTab(Tab::Models));
        update(&mut app, Action::ClickRow(1));
        assert!(app.detail.is_none());
        update(&mut app, Action::ClickRow(1));
        assert_eq!(app.detail.as_ref().unwrap().model, "claude-fable-5");
    }

    #[test]
    fn back_closes_the_popup_before_the_drill_down() {
        let mut app = app();
        update(&mut app, Action::OpenModel(0));
        update(&mut app, Action::OpenBreakdown);
        assert!(app.show_breakdown);
        update(&mut app, Action::Back);
        assert!(!app.show_breakdown);
        assert!(app.detail.is_some());
        update(&mut app, Action::Back);
        assert!(app.detail.is_none());
    }

    #[test]
    fn switching_tabs_closes_the_drill_down() {
        let mut app = app();
        update(&mut app, Action::OpenModel(0));
        update(&mut app, Action::SwitchTab(Tab::Usage));
        assert!(app.detail.is_none());
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
