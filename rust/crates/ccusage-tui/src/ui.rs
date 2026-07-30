//! Rendering: tab bar, report tables, totals footer, and the breakdown popup.
//! Every interactive element registers its screen region in the hit map while
//! it draws, so mouse clicks resolve to the same actions as the keyboard.
use std::collections::HashMap;

use ccusage_core::{
    UsageSummary, format_currency, format_number, format_project_name, short_model_name,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table},
};

use crate::{
    action::Action,
    app::{App, Granularity, Tab},
    data,
    hit::HitMap,
};

pub(crate) fn draw(frame: &mut Frame, app: &mut App, hits: &mut HitMap) {
    hits.clear();
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    draw_header(frame, app, hits, header_area);
    draw_table(frame, app, hits, body_area);
    draw_footer(frame, app, footer_area);
    if app.show_breakdown {
        draw_breakdown(frame, app, hits, frame.area());
    }
}

fn draw_header(frame: &mut Frame, app: &App, hits: &mut HitMap, area: Rect) {
    let [title_area, tabs_area, granularity_area, sort_area] = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Min(0),
        Constraint::Length(10),
        Constraint::Length(7),
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

    let direction = if app.descending() { "desc" } else { "asc" };
    hits.register(sort_area, Action::ToggleSort);
    frame.render_widget(Line::from(format!("[{direction}]").dim()), sort_area);
}

/// The [D][W][M] segmented control; clicking a segment always lands on the
/// Usage tab with that bucketing.
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

    let granularity = app.granularity;
    let period_key = move |row: &UsageSummary| match granularity {
        Granularity::Daily => row.date.clone().unwrap_or_default(),
        Granularity::Weekly => row.week.clone().unwrap_or_default(),
        Granularity::Monthly => row.month.clone().unwrap_or_default(),
    };
    let (header, widths, body_rows) = if app.detail.is_some() {
        period_table(granularity.key_title(), rows, app.hovered_row, period_key)
    } else {
        match app.tab {
            Tab::Usage => period_table(granularity.key_title(), rows, app.hovered_row, period_key),
            Tab::Sessions => session_table(rows, app.hovered_row),
            Tab::Models => models_table(rows, app.hovered_row),
        }
    };
    let table = Table::new(body_rows, widths)
        .header(header.bold())
        .block(block)
        .row_highlight_style(Style::new().reversed());
    frame.render_stateful_widget(table, area, app.state_mut());

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
}

type TableParts = (Row<'static>, Vec<Constraint>, Vec<Row<'static>>);

fn hover_style(hovered: Option<usize>, index: usize) -> Style {
    if hovered == Some(index) {
        Style::new().on_dark_gray()
    } else {
        Style::new()
    }
}

fn period_table(
    key_title: &'static str,
    rows: &[UsageSummary],
    hovered: Option<usize>,
    key: impl Fn(&UsageSummary) -> String,
) -> TableParts {
    let header = Row::new(vec![
        Cell::from(key_title),
        number_cell("Input"),
        number_cell("Output"),
        number_cell("Cache Create"),
        number_cell("Cache Read"),
        number_cell("Total Tokens"),
        number_cell("Cost (USD)"),
        Cell::from("Models"),
    ]);
    let widths = vec![
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(13),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(10),
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
    (header, widths, body)
}

fn models_table(rows: &[UsageSummary], hovered: Option<usize>) -> TableParts {
    let header = Row::new(vec![
        Cell::from("Model"),
        number_cell("Input"),
        number_cell("Output"),
        number_cell("Cache Create"),
        number_cell("Cache Read"),
        number_cell("Total Tokens"),
        number_cell("Cost (USD)"),
        Cell::from("Share"),
    ]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(13),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(10),
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
    (header, widths, body)
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

fn session_table(rows: &[UsageSummary], hovered: Option<usize>) -> TableParts {
    let header = Row::new(vec![
        Cell::from("Project"),
        Cell::from("Session"),
        Cell::from("Last Activity"),
        number_cell("Input"),
        number_cell("Output"),
        number_cell("Total Tokens"),
        number_cell("Cost (USD)"),
    ]);
    let widths = vec![
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Length(13),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(10),
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
    (header, widths, body)
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
        " esc back · [/] switch model · d/w/m bucket · ↑↓/jk move · s sort · enter breakdown"
    } else if app.tab == Tab::Models {
        " q quit · tab/←→ switch · ↑↓/jk move · s sort · enter open model · mouse: click/scroll"
    } else {
        " q quit · tab/←→ switch · d/w/m bucket · ↑↓/jk move · s sort · enter breakdown · mouse: click/scroll"
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
