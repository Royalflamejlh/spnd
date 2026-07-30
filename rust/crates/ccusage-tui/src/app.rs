//! Application state: active tab, report granularity, per-view selection,
//! sort direction, the model drill-down, and the breakdown popup.
use ccusage_core::{UsageSummary, cli::SortOrder};
use ratatui::widgets::TableState;

use crate::data::{self, Tables};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Usage,
    Sessions,
    Models,
}

impl Tab {
    pub(crate) const ALL: [Self; 3] = [Self::Usage, Self::Sessions, Self::Models];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Usage => "Usage",
            Self::Sessions => "Sessions",
            Self::Models => "Models",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::Usage => 0,
            Self::Sessions => 1,
            Self::Models => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Granularity {
    Daily,
    Weekly,
    Monthly,
}

impl Granularity {
    pub(crate) const ALL: [Self; 3] = [Self::Daily, Self::Weekly, Self::Monthly];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Daily => "D",
            Self::Weekly => "W",
            Self::Monthly => "M",
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
        }
    }

    pub(crate) fn key_title(self) -> &'static str {
        match self {
            Self::Daily => "Date",
            Self::Weekly => "Week",
            Self::Monthly => "Month",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Daily => 0,
            Self::Weekly => 1,
            Self::Monthly => 2,
        }
    }
}

/// The drill-down page for one model: its per-period rows at the current
/// granularity plus its own selection and sort direction.
pub(crate) struct ModelDetail {
    /// Index into `tables.models`, which `[`/`]` paging steps through.
    pub(crate) index: usize,
    pub(crate) model: String,
    pub(crate) rows: Vec<UsageSummary>,
    pub(crate) state: TableState,
    descending: bool,
}

/// One selection + sort slot per top-level view: the three usage
/// granularities, the sessions table, and the models table.
const VIEW_COUNT: usize = 5;
const SESSIONS_VIEW: usize = 3;
const MODELS_VIEW: usize = 4;

pub(crate) struct App {
    pub(crate) tables: Tables,
    pub(crate) tab: Tab,
    pub(crate) granularity: Granularity,
    pub(crate) states: [TableState; VIEW_COUNT],
    descending: [bool; VIEW_COUNT],
    pub(crate) detail: Option<ModelDetail>,
    pub(crate) show_breakdown: bool,
    pub(crate) should_quit: bool,
    pub(crate) hovered_row: Option<usize>,
}

impl App {
    pub(crate) fn new(tables: Tables, order: SortOrder) -> Self {
        let date_descending = order == SortOrder::Desc;
        let mut app = Self {
            tables,
            tab: Tab::Usage,
            granularity: Granularity::Daily,
            states: std::array::from_fn(|_| TableState::default()),
            // Sessions and models load most-expensive-first, which reads as
            // descending.
            descending: [
                date_descending,
                date_descending,
                date_descending,
                true,
                true,
            ],
            detail: None,
            show_breakdown: false,
            should_quit: false,
            hovered_row: None,
        };
        for view in 0..VIEW_COUNT {
            let selected = (!app.rows_for(view).is_empty()).then_some(0);
            app.states[view].select(selected);
        }
        app
    }

    /// Index of the active top-level view's selection/sort slot.
    fn view(&self) -> usize {
        match self.tab {
            Tab::Usage => self.granularity.index(),
            Tab::Sessions => SESSIONS_VIEW,
            Tab::Models => MODELS_VIEW,
        }
    }

    pub(crate) fn rows(&self) -> &[UsageSummary] {
        match &self.detail {
            Some(detail) => &detail.rows,
            None => self.rows_for(self.view()),
        }
    }

    fn rows_for(&self, view: usize) -> &[UsageSummary] {
        match view {
            0 => &self.tables.daily,
            1 => &self.tables.weekly,
            2 => &self.tables.monthly,
            SESSIONS_VIEW => &self.tables.sessions,
            _ => &self.tables.models,
        }
    }

    pub(crate) fn state_mut(&mut self) -> &mut TableState {
        let view = self.view();
        match &mut self.detail {
            Some(detail) => &mut detail.state,
            None => &mut self.states[view],
        }
    }

    pub(crate) fn state_selected(&self) -> Option<usize> {
        match &self.detail {
            Some(detail) => detail.state.selected(),
            None => self.states[self.view()].selected(),
        }
    }

    pub(crate) fn selected(&self) -> Option<&UsageSummary> {
        self.rows().get(self.state_selected()?)
    }

    pub(crate) fn descending(&self) -> bool {
        match &self.detail {
            Some(detail) => detail.descending,
            None => self.descending[self.view()],
        }
    }

    pub(crate) fn next_tab(&mut self) {
        self.switch_tab(Tab::ALL[(self.tab.index() + 1) % Tab::ALL.len()]);
    }

    pub(crate) fn prev_tab(&mut self) {
        self.switch_tab(Tab::ALL[(self.tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len()]);
    }

    pub(crate) fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.detail = None;
        self.hovered_row = None;
    }

    /// Selects a usage granularity. With a model detail open this rebuckets
    /// the detail rows in place; otherwise it jumps to the Usage tab so the
    /// shortcut always lands somewhere visible.
    pub(crate) fn set_granularity(&mut self, granularity: Granularity) {
        self.granularity = granularity;
        if self.detail.is_some() {
            self.rebuild_detail();
        } else {
            self.switch_tab(Tab::Usage);
        }
        self.hovered_row = None;
    }

    /// Opens the drill-down for `tables.models[index]`.
    pub(crate) fn open_model(&mut self, index: usize) {
        let Some(row) = self.tables.models.get(index) else {
            return;
        };
        let model = row.models_used.first().cloned().unwrap_or_default();
        self.show_breakdown = false;
        self.tab = Tab::Models;
        self.states[MODELS_VIEW].select(Some(index));
        self.detail = Some(ModelDetail {
            index,
            model,
            rows: Vec::new(),
            state: TableState::default(),
            descending: false,
        });
        self.rebuild_detail();
        self.hovered_row = None;
    }

    pub(crate) fn close_detail(&mut self) {
        self.detail = None;
        self.hovered_row = None;
    }

    /// Steps the open drill-down to the next/previous model, wrapping around.
    pub(crate) fn step_model(&mut self, delta: isize) {
        let count = self.tables.models.len();
        let Some(detail) = &self.detail else {
            return;
        };
        if count == 0 {
            return;
        }
        let index = (detail.index as isize + delta).rem_euclid(count as isize) as usize;
        self.open_model(index);
    }

    fn rebuild_detail(&mut self) {
        let Some(detail) = &mut self.detail else {
            return;
        };
        detail.rows = data::model_series(&self.tables.daily, &detail.model, self.granularity);
        if detail.descending {
            detail.rows.reverse();
        }
        let selected = (!detail.rows.is_empty()).then_some(0);
        detail.state = TableState::default();
        detail.state.select(selected);
    }

    pub(crate) fn move_by(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            return;
        }
        let state = self.state_mut();
        let current = state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, len as isize - 1) as usize;
        state.select(Some(next));
    }

    pub(crate) fn select_first(&mut self) {
        if !self.rows().is_empty() {
            self.state_mut().select(Some(0));
        }
    }

    pub(crate) fn select_last(&mut self) {
        let len = self.rows().len();
        if len > 0 {
            self.state_mut().select(Some(len - 1));
        }
    }

    pub(crate) fn select_row(&mut self, index: usize) {
        if index < self.rows().len() {
            self.state_mut().select(Some(index));
        }
    }

    /// Reverses the active view's rows; every view loads already sorted, so a
    /// reversal is exactly a sort-direction toggle.
    pub(crate) fn toggle_sort(&mut self) {
        if let Some(detail) = &mut self.detail {
            detail.descending ^= true;
            detail.rows.reverse();
            let selected = (!detail.rows.is_empty()).then_some(0);
            detail.state.select(selected);
        } else {
            let view = self.view();
            self.descending[view] ^= true;
            match view {
                0 => self.tables.daily.reverse(),
                1 => self.tables.weekly.reverse(),
                2 => self.tables.monthly.reverse(),
                SESSIONS_VIEW => self.tables.sessions.reverse(),
                _ => self.tables.models.reverse(),
            }
            self.select_first();
        }
        self.hovered_row = None;
    }
}
