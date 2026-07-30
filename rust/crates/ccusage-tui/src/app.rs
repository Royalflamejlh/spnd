//! Application state: active tab, report granularity, per-view selection and
//! sort, the model drill-down, and the popups.
use std::cmp::Ordering;

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

/// A sortable table column. `Key` is whatever identifies the row in its view:
/// the period for usage tables, project + session for sessions, the model
/// name for models.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SortColumn {
    Key,
    Activity,
    Input,
    Output,
    CacheCreate,
    CacheRead,
    TotalTokens,
    Cost,
}

impl SortColumn {
    /// Numeric columns read best largest-first; textual ones smallest-first.
    fn default_descending(self) -> bool {
        !matches!(self, Self::Key | Self::Activity)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Sort {
    pub(crate) column: SortColumn,
    pub(crate) descending: bool,
}

pub(crate) const PERIOD_SORT_COLUMNS: [SortColumn; 7] = [
    SortColumn::Key,
    SortColumn::Input,
    SortColumn::Output,
    SortColumn::CacheCreate,
    SortColumn::CacheRead,
    SortColumn::TotalTokens,
    SortColumn::Cost,
];
pub(crate) const SESSION_SORT_COLUMNS: [SortColumn; 6] = [
    SortColumn::Key,
    SortColumn::Activity,
    SortColumn::Input,
    SortColumn::Output,
    SortColumn::TotalTokens,
    SortColumn::Cost,
];

/// The row's identity for `SortColumn::Key` ordering, uniform across views.
pub(crate) fn key_pair(row: &UsageSummary) -> (&str, &str) {
    let first = row
        .date
        .as_deref()
        .or(row.week.as_deref())
        .or(row.month.as_deref())
        .or(row.project_path.as_deref())
        .or_else(|| row.models_used.first().map(String::as_str))
        .unwrap_or_default();
    (first, row.session_id.as_deref().unwrap_or_default())
}

fn compare_rows(a: &UsageSummary, b: &UsageSummary, column: SortColumn) -> Ordering {
    match column {
        SortColumn::Key => key_pair(a).cmp(&key_pair(b)),
        SortColumn::Activity => a.last_activity.cmp(&b.last_activity),
        SortColumn::Input => a.input_tokens.cmp(&b.input_tokens),
        SortColumn::Output => a.output_tokens.cmp(&b.output_tokens),
        SortColumn::CacheCreate => a.cache_creation_tokens.cmp(&b.cache_creation_tokens),
        SortColumn::CacheRead => a.cache_read_tokens.cmp(&b.cache_read_tokens),
        SortColumn::TotalTokens => a.total_tokens().cmp(&b.total_tokens()),
        SortColumn::Cost => a.total_cost.total_cmp(&b.total_cost),
    }
}

fn sort_rows(rows: &mut [UsageSummary], sort: Sort) {
    rows.sort_by(|a, b| {
        let ordering = compare_rows(a, b, sort.column);
        if sort.descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

/// The drill-down page for one model: its per-period rows at the current
/// granularity plus its own selection and sort.
pub(crate) struct ModelDetail {
    /// Index into `tables.models`, which `[`/`]` paging steps through.
    pub(crate) index: usize,
    pub(crate) model: String,
    pub(crate) rows: Vec<UsageSummary>,
    pub(crate) state: TableState,
    sort: Sort,
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
    sorts: [Sort; VIEW_COUNT],
    pub(crate) detail: Option<ModelDetail>,
    pub(crate) show_breakdown: bool,
    pub(crate) show_help: bool,
    pub(crate) should_quit: bool,
    pub(crate) hovered_row: Option<usize>,
}

impl App {
    pub(crate) fn new(tables: Tables, order: SortOrder) -> Self {
        let by_period = Sort {
            column: SortColumn::Key,
            descending: order == SortOrder::Desc,
        };
        // Sessions and models load most-expensive-first.
        let by_cost = Sort {
            column: SortColumn::Cost,
            descending: true,
        };
        let mut app = Self {
            tables,
            tab: Tab::Usage,
            granularity: Granularity::Daily,
            states: std::array::from_fn(|_| TableState::default()),
            sorts: [by_period, by_period, by_period, by_cost, by_cost],
            detail: None,
            show_breakdown: false,
            show_help: false,
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

    fn active_rows_mut(&mut self) -> &mut Vec<UsageSummary> {
        let view = self.view();
        match &mut self.detail {
            Some(detail) => &mut detail.rows,
            None => match view {
                0 => &mut self.tables.daily,
                1 => &mut self.tables.weekly,
                2 => &mut self.tables.monthly,
                SESSIONS_VIEW => &mut self.tables.sessions,
                _ => &mut self.tables.models,
            },
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

    pub(crate) fn sort(&self) -> Sort {
        match &self.detail {
            Some(detail) => detail.sort,
            None => self.sorts[self.view()],
        }
    }

    /// The sortable columns of the active view, in cycling order.
    pub(crate) fn sort_columns(&self) -> &'static [SortColumn] {
        if self.detail.is_some() {
            return &PERIOD_SORT_COLUMNS;
        }
        match self.tab {
            Tab::Usage | Tab::Models => &PERIOD_SORT_COLUMNS,
            Tab::Sessions => &SESSION_SORT_COLUMNS,
        }
    }

    fn set_sort(&mut self, sort: Sort) {
        match &mut self.detail {
            Some(detail) => detail.sort = sort,
            None => {
                let view = self.view();
                self.sorts[view] = sort;
            }
        }
        sort_rows(self.active_rows_mut(), sort);
        self.hovered_row = None;
        self.select_first();
    }

    pub(crate) fn toggle_sort(&mut self) {
        let mut sort = self.sort();
        sort.descending = !sort.descending;
        self.set_sort(sort);
    }

    /// Sorts by the given column: a repeat on the active column flips the
    /// direction, a new column starts at that column's natural direction.
    pub(crate) fn sort_by(&mut self, column: SortColumn) {
        let current = self.sort();
        let descending = if current.column == column {
            !current.descending
        } else {
            column.default_descending()
        };
        self.set_sort(Sort { column, descending });
    }

    /// Steps the sort column through the active view's column list.
    pub(crate) fn cycle_sort_column(&mut self) {
        let columns = self.sort_columns();
        let current = self.sort().column;
        let position = columns.iter().position(|&column| column == current);
        let next = columns[position.map_or(0, |index| (index + 1) % columns.len())];
        self.set_sort(Sort {
            column: next,
            descending: next.default_descending(),
        });
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
            sort: Sort {
                column: SortColumn::Key,
                descending: false,
            },
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
        sort_rows(&mut detail.rows, detail.sort);
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
}
