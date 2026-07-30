//! Rendering: tab bar, chart strip, report tables, totals footer, and the
//! popups. Every interactive element registers its screen region in the hit
//! map while it draws, so mouse clicks resolve to the same actions as the
//! keyboard.
use std::collections::HashMap;

use ccusage_core::{
    UsageSummary, format_currency, format_number, format_project_name, short_model_name,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Bar, BarChart, Block, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
};

use crate::{
    action::Action,
    app::{App, Granularity, Sort, SortColumn, Tab, key_pair},
    data,
    hit::HitMap,
};

const CHART_HEIGHT: u16 = 8;
const BAR_WIDTH: u16 = 4;
const BAR_GAP: u16 = 1;

pub(crate) fn draw(frame: &mut Frame, app: &mut App, hits: &mut HitMap) {
    hits.clear();
    let [header_area, chart_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(chart_height(frame.area(), app)),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    draw_header(frame, app, hits, header_area);
    if chart_area.height > 0 {
        draw_chart(frame, app, hits, chart_area);
    }
    draw_table(frame, app, hits, body_area);
    draw_footer(frame, app, footer_area);
    if app.show_breakdown {
        draw_breakdown(frame, app, hits, frame.area());
    }
    if app.show_help {
        draw_help(frame, frame.area());
    }
}

/// The chart strip only makes sense for bucketed views with something to
/// compare, and only when the terminal leaves enough room for the table.
fn chart_height(area: Rect, app: &App) -> u16 {
    let bucketed = app.tab == Tab::Usage || app.detail.is_some();
    if bucketed && area.height >= 20 && app.rows().len() > 1 {
        CHART_HEIGHT
    } else {
        0
    }
}

fn draw_header(frame: &mut Frame, app: &App, hits: &mut HitMap, area: Rect) {
    let [title_area, tabs_area, granularity_area, sort_area] = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Min(0),
        Constraint::Length(10),
        Constraint::Length(12),
    ])
    .areas(area);

    frame.render_widget(Line::from(" ccusage ".bold().cyan()), title_area);

    let mut spans = Vec::new();
    let mut x = tabs_area.x;
    for (index, tab) in Tab::ALL.into_iter().enumerate() {
        if index > 0 {
            spans.push("│".dim());
            x += 1;
        }
        let label = format!(" {} ", tab.title());
        let width = label.len() as u16;
        hits.register(Rect::new(x, tabs_area.y, width, 1), Action::SwitchTab(tab));
        spans.push(if tab == app.tab {
            label.bold().cyan().underlined()
        } else {
            Span::from(label)
        });
        x += width;
    }
    frame.render_widget(Line::from(spans), tabs_area);

    draw_granularity(frame, app, hits, granularity_area);

    let sort = app.sort();
    let indicator = format!(
        "[{}{}]",
        sort.column.short_label(),
        if sort.descending { "▾" } else { "▴" }
    );
    hits.register(sort_area, Action::ToggleSort);
    frame.render_widget(Line::from(indicator).right_aligned().dim(), sort_area);
}

/// The [D][W][M] segmented control; clicking a segment rebuckets the open
/// drill-down, or lands on the Usage tab with that bucketing.
fn draw_granularity(frame: &mut Frame, app: &App, hits: &mut HitMap, area: Rect) {
    let mut spans = Vec::new();
    let mut x = area.x;
    for granularity in Granularity::ALL {
        let label = format!("[{}]", granularity.label());
        let width = label.len() as u16;
        hits.register(
            Rect::new(x, area.y, width, 1),
            Action::SetGranularity(granularity),
        );
        let bucketed_view = app.tab == Tab::Usage || app.detail.is_some();
        spans.push(if bucketed_view && granularity == app.granularity {
            label.bold().cyan()
        } else {
            label.dim()
        });
        x += width;
    }
    frame.render_widget(Line::from(spans), area);
}

/// A clickable bar chart of cost per bucket, chronological left to right,
/// clipped to the most recent buckets that fit.
fn draw_chart(frame: &mut Frame, app: &App, hits: &mut HitMap, area: Rect) {
    let rows = app.rows();
    let mut items: Vec<(usize, &UsageSummary)> = rows.iter().enumerate().collect();
    items.sort_by(|a, b| key_pair(a.1).cmp(&key_pair(b.1)));
    let fit = usize::from(area.width.saturating_sub(2) / (BAR_WIDTH + BAR_GAP));
    if fit == 0 {
        return;
    }
    let start = items.len().saturating_sub(fit);
    let items = &items[start..];

    let selected = app.state_selected();
    let bars: Vec<Bar> = items
        .iter()
        .map(|(row_index, row)| {
            let key = key_pair(row).0;
            let label = key.get(key.len().saturating_sub(2)..).unwrap_or(key);
            Bar::default()
                .value((row.total_cost * 100.0).round() as u64)
                .label(Line::from(label.to_string()))
                .text_value(String::new())
                .style(if selected == Some(*row_index) {
                    Style::new().cyan().bold()
                } else {
                    Style::new().cyan().dim()
                })
        })
        .collect();
    let hidden = start;
    let mut block = Block::bordered()
        .title(format!(
            " cost per {} ",
            app.granularity.title().to_lowercase()
        ))
        .dim();
    if hidden > 0 {
        block = block.title_top(
            Line::from(format!(" {hidden} earlier hidden "))
                .right_aligned()
                .dim(),
        );
    }
    let chart = BarChart::new(bars)
        .bar_width(BAR_WIDTH)
        .bar_gap(BAR_GAP)
        .block(block);
    frame.render_widget(chart, area);

    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    for (position, (row_index, _)) in items.iter().enumerate() {
        let x = inner.x + position as u16 * (BAR_WIDTH + BAR_GAP);
        hits.register(
            Rect::new(x, inner.y, BAR_WIDTH, inner.height),
            Action::SelectRow(*row_index),
        );
    }
}

fn draw_table(frame: &mut Frame, app: &mut App, hits: &mut HitMap, area: Rect) {
    let rows = app.rows();
    let row_count = rows.len();
    let plural = if row_count == 1 { "row" } else { "rows" };
    let title = match &app.detail {
        Some(detail) => format!(
            " {} — {} — {row_count} {plural} ",
            short_model_name(&detail.model),
            app.granularity.title(),
        ),
        None => {
            let title = match app.tab {
                Tab::Usage => app.granularity.title(),
                Tab::Sessions | Tab::Models => app.tab.title(),
            };
            format!(" {title} — {row_count} {plural} ")
        }
    };
    let mut block = Block::bordered().title(title).dim();
    if app.detail.is_some() {
        block = block.title_bottom(Line::from(" [/] switch model · esc back ").right_aligned());
    }
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No rows for this report.").block(block),
            area,
        );
        return;
    }

    let sort = app.sort();
    let granularity = app.granularity;
    let period_key = move |row: &UsageSummary| match granularity {
        Granularity::Daily => row.date.clone().unwrap_or_default(),
        Granularity::Weekly => row.week.clone().unwrap_or_default(),
        Granularity::Monthly => row.month.clone().unwrap_or_default(),
    };
    let (header, widths, body_rows, sort_map) = if app.detail.is_some() {
        period_table(
            granularity.key_title(),
            rows,
            app.hovered_row,
            sort,
            period_key,
        )
    } else {
        match app.tab {
            Tab::Usage => period_table(
                granularity.key_title(),
                rows,
                app.hovered_row,
                sort,
                period_key,
            ),
            Tab::Sessions => session_table(rows, app.hovered_row, sort),
            Tab::Models => models_table(rows, app.hovered_row, sort),
        }
    };
    let table = Table::new(body_rows, widths.clone())
        .header(header.bold())
        .block(block)
        .row_highlight_style(Style::new().reversed());
    frame.render_stateful_widget(table, area, app.state_mut());

    // Column headers sort on click; resolve their rects with the same layout
    // the table itself uses.
    let header_row = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), 1);
    let column_areas = Layout::horizontal(widths).spacing(1).split(header_row);
    for (rect, column) in column_areas.iter().zip(sort_map) {
        if let Some(column) = column {
            hits.register(*rect, Action::SortBy(*column));
        }
    }

    // The stateful render just fixed up the scroll offset, so the visible
    // window is only known now: one row per line under the border + header.
    let offset = app.state_mut().offset();
    let body_top = area.y + 2;
    let body_height = usize::from(area.height.saturating_sub(3));
    let visible = row_count.saturating_sub(offset).min(body_height);
    for index in 0..visible {
        hits.register(
            Rect::new(
                area.x + 1,
                body_top + index as u16,
                area.width.saturating_sub(2),
                1,
            ),
            Action::ClickRow(offset + index),
        );
    }

    if row_count > body_height {
        let track = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });
        let mut state = ScrollbarState::new(row_count).position(offset);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            track,
            &mut state,
        );
        let track = Rect::new(track.right().saturating_sub(1), track.y, 1, track.height);
        hits.set_scrollbar(track, row_count);
    }
}

impl SortColumn {
    fn short_label(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Activity => "act",
            Self::Input => "in",
            Self::Output => "out",
            Self::CacheCreate => "cc",
            Self::CacheRead => "cr",
            Self::TotalTokens => "tok",
            Self::Cost => "cost",
        }
    }
}

type TableParts = (
    Row<'static>,
    Vec<Constraint>,
    Vec<Row<'static>>,
    &'static [Option<SortColumn>],
);

fn hover_style(hovered: Option<usize>, index: usize) -> Style {
    if hovered == Some(index) {
        Style::new().on_dark_gray()
    } else {
        Style::new()
    }
}

/// A header cell, marked with the sort direction when it is the active sort
/// column.
fn header_cell(
    title: &str,
    column: Option<SortColumn>,
    sort: Sort,
    numeric: bool,
) -> Cell<'static> {
    let text = match column {
        Some(column) if column == sort.column => {
            format!("{title} {}", if sort.descending { "▾" } else { "▴" })
        }
        _ => title.to_string(),
    };
    if numeric {
        number_cell(text)
    } else {
        Cell::from(text)
    }
}

const PERIOD_SORT_MAP: [Option<SortColumn>; 8] = [
    Some(SortColumn::Key),
    Some(SortColumn::Input),
    Some(SortColumn::Output),
    Some(SortColumn::CacheCreate),
    Some(SortColumn::CacheRead),
    Some(SortColumn::TotalTokens),
    Some(SortColumn::Cost),
    None,
];

fn period_table(
    key_title: &'static str,
    rows: &[UsageSummary],
    hovered: Option<usize>,
    sort: Sort,
    key: impl Fn(&UsageSummary) -> String,
) -> TableParts {
    let header = Row::new(vec![
        header_cell(key_title, Some(SortColumn::Key), sort, false),
        header_cell("Input", Some(SortColumn::Input), sort, true),
        header_cell("Output", Some(SortColumn::Output), sort, true),
        header_cell("Cache Create", Some(SortColumn::CacheCreate), sort, true),
        header_cell("Cache Read", Some(SortColumn::CacheRead), sort, true),
        header_cell("Total Tokens", Some(SortColumn::TotalTokens), sort, true),
        header_cell("Cost (USD)", Some(SortColumn::Cost), sort, true),
        Cell::from("Models"),
    ]);
    let widths = vec![
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Fill(1),
    ];
    let body = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(key(row)),
                number_cell(format_number(row.input_tokens)),
                number_cell(format_number(row.output_tokens)),
                number_cell(format_number(row.cache_creation_tokens)),
                number_cell(format_number(row.cache_read_tokens)),
                number_cell(format_number(row.total_tokens())),
                cost_cell(row.total_cost),
                Cell::from(model_list(row)),
            ])
            .style(hover_style(hovered, index))
        })
        .collect();
    (header, widths, body, &PERIOD_SORT_MAP)
}

const MODELS_SORT_MAP: [Option<SortColumn>; 8] = [
    Some(SortColumn::Key),
    Some(SortColumn::Input),
    Some(SortColumn::Output),
    Some(SortColumn::CacheCreate),
    Some(SortColumn::CacheRead),
    Some(SortColumn::TotalTokens),
    Some(SortColumn::Cost),
    Some(SortColumn::Cost),
];

fn models_table(rows: &[UsageSummary], hovered: Option<usize>, sort: Sort) -> TableParts {
    let header = Row::new(vec![
        header_cell("Model", Some(SortColumn::Key), sort, false),
        header_cell("Input", Some(SortColumn::Input), sort, true),
        header_cell("Output", Some(SortColumn::Output), sort, true),
        header_cell("Cache Create", Some(SortColumn::CacheCreate), sort, true),
        header_cell("Cache Read", Some(SortColumn::CacheRead), sort, true),
        header_cell("Total Tokens", Some(SortColumn::TotalTokens), sort, true),
        header_cell("Cost (USD)", Some(SortColumn::Cost), sort, true),
        Cell::from("Share"),
    ]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(18),
    ];
    let total_cost: f64 = rows.iter().map(|row| row.total_cost).sum();
    let body = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(
                    row.models_used
                        .first()
                        .map(|model| short_model_name(model))
                        .unwrap_or_default(),
                ),
                number_cell(format_number(row.input_tokens)),
                number_cell(format_number(row.output_tokens)),
                number_cell(format_number(row.cache_creation_tokens)),
                number_cell(format_number(row.cache_read_tokens)),
                number_cell(format_number(row.total_tokens())),
                cost_cell(row.total_cost),
                share_cell(row.total_cost, total_cost),
            ])
            .style(hover_style(hovered, index))
        })
        .collect();
    (header, widths, body, &MODELS_SORT_MAP)
}

/// A percentage plus a proportional bar, e.g. ` 42% █████`.
fn share_cell(cost: f64, total: f64) -> Cell<'static> {
    const BAR_WIDTH: usize = 12;
    let fraction = if total > 0.0 { cost / total } else { 0.0 };
    let filled = (fraction * BAR_WIDTH as f64).round() as usize;
    let bar = "█".repeat(filled.min(BAR_WIDTH));
    Cell::from(Line::from(vec![
        Span::from(format!("{:>3.0}% ", fraction * 100.0)),
        bar.cyan(),
    ]))
}

const SESSION_SORT_MAP: [Option<SortColumn>; 7] = [
    Some(SortColumn::Key),
    Some(SortColumn::Key),
    Some(SortColumn::Activity),
    Some(SortColumn::Input),
    Some(SortColumn::Output),
    Some(SortColumn::TotalTokens),
    Some(SortColumn::Cost),
];

fn session_table(rows: &[UsageSummary], hovered: Option<usize>, sort: Sort) -> TableParts {
    let header = Row::new(vec![
        header_cell("Project", Some(SortColumn::Key), sort, false),
        header_cell("Session", Some(SortColumn::Key), sort, false),
        header_cell("Last Activity", Some(SortColumn::Activity), sort, false),
        header_cell("Input", Some(SortColumn::Input), sort, true),
        header_cell("Output", Some(SortColumn::Output), sort, true),
        header_cell("Total Tokens", Some(SortColumn::TotalTokens), sort, true),
        header_cell("Cost (USD)", Some(SortColumn::Cost), sort, true),
    ]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Length(15),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(12),
    ];
    let aliases = HashMap::new();
    let body = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(format_project_name(
                    row.project_path.as_deref().unwrap_or_default(),
                    &aliases,
                )),
                Cell::from(row.session_id.clone().unwrap_or_default()),
                Cell::from(activity_date(row)),
                number_cell(format_number(row.input_tokens)),
                number_cell(format_number(row.output_tokens)),
                number_cell(format_number(row.total_tokens())),
                cost_cell(row.total_cost),
            ])
            .style(hover_style(hovered, index))
        })
        .collect();
    (header, widths, body, &SESSION_SORT_MAP)
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let [totals_area, hints_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
    let totals = data::totals(app.rows());
    let line = Line::from(vec![
        " Totals ".bold(),
        Span::from(format!(
            " In {}  Out {}  Cache {}/{}  Tokens {}  ",
            format_number(totals.input_tokens),
            format_number(totals.output_tokens),
            format_number(totals.cache_creation_tokens),
            format_number(totals.cache_read_tokens),
            format_number(totals.total_tokens),
        )),
        format_currency(totals.cost).bold().yellow(),
    ]);
    frame.render_widget(line, totals_area);
    let hints = if app.detail.is_some() {
        " esc back · [/] switch model · d/w/m bucket · ↑↓ move · s/o sort · enter breakdown · ? help"
    } else if app.tab == Tab::Models {
        " q quit · tab switch · ↑↓ move · s/o sort · enter open model · ? help"
    } else {
        " q quit · tab switch · d/w/m bucket · ↑↓ move · s/o sort · enter breakdown · ? help"
    };
    frame.render_widget(Line::from(hints).dim(), hints_area);
}

fn draw_breakdown(frame: &mut Frame, app: &App, hits: &mut HitMap, area: Rect) {
    let Some(row) = app.selected() else {
        return;
    };
    let height = (row.model_breakdowns.len() as u16 + 4).min(area.height.saturating_sub(2));
    let popup = centered_rect(area, 88, height);
    hits.set_popup(popup);
    frame.render_widget(Clear, popup);

    let header = Row::new(vec![
        Cell::from("Model"),
        number_cell("Input"),
        number_cell("Output"),
        number_cell("Cache Create"),
        number_cell("Cache Read"),
        number_cell("Cost (USD)"),
    ]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(13),
        Constraint::Length(14),
        Constraint::Length(10),
    ];
    let body = row
        .model_breakdowns
        .iter()
        .map(|breakdown| {
            Row::new(vec![
                Cell::from(short_model_name(&breakdown.model_name)),
                number_cell(format_number(breakdown.input_tokens)),
                number_cell(format_number(breakdown.output_tokens)),
                number_cell(format_number(breakdown.cache_creation_tokens)),
                number_cell(format_number(breakdown.cache_read_tokens)),
                cost_cell(breakdown.cost),
            ])
        })
        .collect::<Vec<_>>();
    // Each popup row is a jump to that model's drill-down.
    for (index, breakdown) in row.model_breakdowns.iter().enumerate() {
        let Some(model_index) = app
            .tables
            .models
            .iter()
            .position(|model| model.models_used.first() == Some(&breakdown.model_name))
        else {
            continue;
        };
        hits.register_popup(
            Rect::new(
                popup.x + 1,
                popup.y + 2 + index as u16,
                popup.width.saturating_sub(2),
                1,
            ),
            Action::OpenModel(model_index),
        );
    }
    let table = Table::new(body, widths).header(header.bold()).block(
        Block::bordered()
            .title(format!(" {} — model breakdown ", row_key(row)))
            .title_bottom(
                Line::from(" click a model to drill in · esc close ")
                    .right_aligned()
                    .dim(),
            ),
    );
    frame.render_widget(table, popup);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = [
        ("tab / ← →", "switch between Usage, Sessions, Models"),
        ("d / w / m", "daily, weekly, or monthly bucketing"),
        ("↑ ↓ / j k", "move the selection (PgUp/PgDn: ten rows)"),
        ("g / G", "jump to the first / last row"),
        ("s", "flip the sort direction"),
        ("o", "sort by the next column"),
        ("enter / b", "model breakdown (Models tab: drill in)"),
        ("[ / ]", "previous / next model in the drill-down"),
        ("esc / backspace", "back out of a popup or drill-down"),
        ("q / ctrl-c", "quit"),
        ("", ""),
        ("mouse", "click tabs, rows, headers, bars, [D][W][M];"),
        (
            "",
            "wheel scrolls, drag the scrollbar, right-click backs out",
        ),
    ]
    .into_iter()
    .map(|(keys, effect)| {
        Line::from(vec![
            Span::from(format!(" {keys:>15}  ")).bold().cyan(),
            Span::from(effect),
        ])
    })
    .collect();
    let height = (lines.len() as u16 + 2).min(area.height.saturating_sub(2));
    let popup = centered_rect(area, 72, height);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title(" keys ")
                .title_bottom(Line::from(" any key closes ").right_aligned().dim()),
        ),
        popup,
    );
}

fn row_key(row: &UsageSummary) -> String {
    row.date
        .clone()
        .or_else(|| row.week.clone())
        .or_else(|| row.month.clone())
        .or_else(|| row.session_id.clone())
        .unwrap_or_default()
}

fn activity_date(row: &UsageSummary) -> String {
    let activity = row.last_activity.as_deref().unwrap_or_default();
    activity.get(..10).unwrap_or(activity).to_string()
}

fn model_list(row: &UsageSummary) -> String {
    row.models_used
        .iter()
        .map(|model| short_model_name(model))
        .collect::<Vec<_>>()
        .join(", ")
}

fn number_cell(value: impl Into<String>) -> Cell<'static> {
    Cell::from(Line::from(value.into()).right_aligned())
}

fn cost_cell(value: f64) -> Cell<'static> {
    Cell::from(Line::from(format_currency(value)).right_aligned().yellow())
}

fn centered_rect(area: Rect, width_percent: u16, height: u16) -> Rect {
    let width = (u32::from(area.width) * u32::from(width_percent) / 100) as u16;
    let width = width.min(area.width.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
