//! Application state: active tab, per-tab selection, sort direction, and the
//! model breakdown popup.
use ccusage_core::{UsageSummary, cli::SortOrder};
use ratatui::widgets::TableState;

use crate::data::Tables;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Daily,
    Monthly,
    Sessions,
}

impl Tab {
    pub(crate) const ALL: [Self; 3] = [Self::Daily, Self::Monthly, Self::Sessions];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Monthly => "Monthly",
            Self::Sessions => "Sessions",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::Daily => 0,
            Self::Monthly => 1,
            Self::Sessions => 2,
        }
    }
}

pub(crate) struct App {
    pub(crate) tables: Tables,
    pub(crate) tab: Tab,
    pub(crate) states: [TableState; 3],
    descending: [bool; 3],
    pub(crate) show_breakdown: bool,
    pub(crate) should_quit: bool,
    pub(crate) hovered_row: Option<usize>,
}

impl App {
    pub(crate) fn new(tables: Tables, order: SortOrder) -> Self {
        let date_descending = order == SortOrder::Desc;
        let mut app = Self {
            tables,
            tab: Tab::Daily,
            states: [
                TableState::default(),
                TableState::default(),
                TableState::default(),
            ],
            // Sessions load most-expensive-first, which reads as descending.
            descending: [date_descending, date_descending, true],
            show_breakdown: false,
            should_quit: false,
            hovered_row: None,
        };
        for tab in Tab::ALL {
            let selected = (!app.rows_for(tab).is_empty()).then_some(0);
            app.states[tab.index()].select(selected);
        }
        app
    }

    pub(crate) fn rows(&self) -> &[UsageSummary] {
        self.rows_for(self.tab)
    }

    fn rows_for(&self, tab: Tab) -> &[UsageSummary] {
        match tab {
            Tab::Daily => &self.tables.daily,
            Tab::Monthly => &self.tables.monthly,
            Tab::Sessions => &self.tables.sessions,
        }
    }

    fn rows_mut(&mut self) -> &mut Vec<UsageSummary> {
        match self.tab {
            Tab::Daily => &mut self.tables.daily,
            Tab::Monthly => &mut self.tables.monthly,
            Tab::Sessions => &mut self.tables.sessions,
        }
    }

    pub(crate) fn state_mut(&mut self) -> &mut TableState {
        &mut self.states[self.tab.index()]
    }

    pub(crate) fn selected(&self) -> Option<&UsageSummary> {
        let selected = self.states[self.tab.index()].selected()?;
        self.rows().get(selected)
    }

    pub(crate) fn descending(&self) -> bool {
        self.descending[self.tab.index()]
    }

    pub(crate) fn state_selected(&self) -> Option<usize> {
        self.states[self.tab.index()].selected()
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

    /// Reverses the current tab's rows; every tab loads already sorted, so a
    /// reversal is exactly a sort-direction toggle.
    pub(crate) fn toggle_sort(&mut self) {
        self.descending[self.tab.index()] ^= true;
        self.rows_mut().reverse();
        self.hovered_row = None;
        self.select_first();
    }
}
