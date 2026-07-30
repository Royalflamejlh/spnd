//! Application state: active tab, report granularity, per-view selection,
//! sort direction, and the model breakdown popup.
use ccusage_core::{UsageSummary, cli::SortOrder};
use ratatui::widgets::TableState;

use crate::data::Tables;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Usage,
    Sessions,
}

impl Tab {
    pub(crate) const ALL: [Self; 2] = [Self::Usage, Self::Sessions];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Usage => "Usage",
            Self::Sessions => "Sessions",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::Usage => 0,
            Self::Sessions => 1,
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

/// One selection + sort slot per view: the three usage granularities plus the
/// sessions table.
const VIEW_COUNT: usize = 4;
const SESSIONS_VIEW: usize = 3;

pub(crate) struct App {
    pub(crate) tables: Tables,
    pub(crate) tab: Tab,
    pub(crate) granularity: Granularity,
    pub(crate) states: [TableState; VIEW_COUNT],
    descending: [bool; VIEW_COUNT],
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
            // Sessions load most-expensive-first, which reads as descending.
            descending: [date_descending, date_descending, date_descending, true],
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

    /// Index of the active view's selection/sort slot.
    fn view(&self) -> usize {
        match self.tab {
            Tab::Usage => self.granularity.index(),
            Tab::Sessions => SESSIONS_VIEW,
        }
    }

    pub(crate) fn rows(&self) -> &[UsageSummary] {
        self.rows_for(self.view())
    }

    fn rows_for(&self, view: usize) -> &[UsageSummary] {
        match view {
            0 => &self.tables.daily,
            1 => &self.tables.weekly,
            2 => &self.tables.monthly,
            _ => &self.tables.sessions,
        }
    }

    fn rows_mut(&mut self) -> &mut Vec<UsageSummary> {
        match self.view() {
            0 => &mut self.tables.daily,
            1 => &mut self.tables.weekly,
            2 => &mut self.tables.monthly,
            _ => &mut self.tables.sessions,
        }
    }

    pub(crate) fn state_mut(&mut self) -> &mut TableState {
        &mut self.states[self.view()]
    }

    pub(crate) fn state_selected(&self) -> Option<usize> {
        self.states[self.view()].selected()
    }

    pub(crate) fn selected(&self) -> Option<&UsageSummary> {
        self.rows().get(self.state_selected()?)
    }

    pub(crate) fn descending(&self) -> bool {
        self.descending[self.view()]
    }

    pub(crate) fn next_tab(&mut self) {
        self.switch_tab(Tab::ALL[(self.tab.index() + 1) % Tab::ALL.len()]);
    }

    pub(crate) fn prev_tab(&mut self) {
        self.switch_tab(Tab::ALL[(self.tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len()]);
    }

    pub(crate) fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.hovered_row = None;
    }

    /// Selects a usage granularity; from any other tab this also jumps to the
    /// Usage tab so the shortcut always lands somewhere visible.
    pub(crate) fn set_granularity(&mut self, granularity: Granularity) {
        self.granularity = granularity;
        self.switch_tab(Tab::Usage);
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

    /// Reverses the current view's rows; every view loads already sorted, so a
    /// reversal is exactly a sort-direction toggle.
    pub(crate) fn toggle_sort(&mut self) {
        let view = self.view();
        self.descending[view] ^= true;
        self.rows_mut().reverse();
        self.hovered_row = None;
        self.select_first();
    }
}
