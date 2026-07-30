//! Rendering: tab bar, report tables, totals footer, and the breakdown popup.
use std::collections::HashMap;

use ccusage_core::{
    UsageSummary, format_currency, format_number, format_project_name, short_model_name,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table, Tabs},
};

use crate::{
    app::{App, Tab},
    data,
};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    draw_header(frame, app, header_area);
    draw_table(frame, app, body_area);
    draw_footer(frame, app, footer_area);
    if app.show_breakdown {
        draw_breakdown(frame, app, frame.area());
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [title_area, tabs_area, sort_area] = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Min(0),
        Constraint::Length(7),
    ])
    .areas(area);

    frame.render_widget(Line::from(" ccusage ".bold().cyan()), title_area);
    let tabs = Tabs::new(Tab::ALL.map(Tab::title))
        .select(app.tab.index())
        .highlight_style(Style::new().bold().cyan().underlined());
    frame.render_widget(tabs, tabs_area);
    let direction = if app.descending() { "desc" } else { "asc" };
    frame.render_widget(Line::from(format!("[{direction}]").dim()), sort_area);
}

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows = app.rows();
    let plural = if rows.len() == 1 { "row" } else { "rows" };
    let block = Block::bordered()
        .title(format!(" {} — {} {plural} ", app.tab.title(), rows.len()))
        .dim();
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No rows for this report.").block(block),
            area,
        );
        return;
    }

    let (header, widths, body_rows) = match app.tab {
        Tab::Daily => period_table("Date", rows, |row| row.date.clone().unwrap_or_default()),
        Tab::Monthly => period_table("Month", rows, |row| row.month.clone().unwrap_or_default()),
        Tab::Sessions => session_table(rows),
    };
    let table = Table::new(body_rows, widths)
        .header(header.bold())
        .block(block)
        .row_highlight_style(Style::new().reversed());
    frame.render_stateful_widget(table, area, app.state_mut());
}

type TableParts = (Row<'static>, Vec<Constraint>, Vec<Row<'static>>);

fn period_table(
    key_title: &'static str,
    rows: &[UsageSummary],
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
        .map(|row| {
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
        })
        .collect();
    (header, widths, body)
}

fn session_table(rows: &[UsageSummary]) -> TableParts {
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
        .map(|row| {
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
    frame.render_widget(
        Line::from(" q quit · tab/←→ switch · ↑↓/jk move · g/G ends · s sort · enter breakdown")
            .dim(),
        hints_area,
    );
}

fn draw_breakdown(frame: &mut Frame, app: &App, area: Rect) {
    let Some(row) = app.selected() else {
        return;
    };
    let height = (row.model_breakdowns.len() as u16 + 4).min(area.height.saturating_sub(2));
    let popup = centered_rect(area, 88, height);
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
    let table = Table::new(body, widths).header(header.bold()).block(
        Block::bordered()
            .title(format!(" {} — model breakdown ", row_key(row)))
            .title_bottom(Line::from(" esc close ").right_aligned().dim()),
    );
    frame.render_widget(table, popup);
}

fn row_key(row: &UsageSummary) -> String {
    row.date
        .clone()
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
