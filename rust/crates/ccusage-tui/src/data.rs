//! Shapes loaded usage entries into the rows each tab displays.
use std::sync::Arc;

use ccusage_core::{
    BucketKind, LoadedEntry, ModelBreakdown, Result, SessionAccumulator, UsageSummary,
    cli::{SharedArgs, WeekDay},
    fast::FxHashMap,
    filter_and_sort_summaries, sort_summaries, summarize_by_key, summarize_summaries_by_bucket,
};

use crate::app::Granularity;

pub(crate) struct Tables {
    pub(crate) daily: Vec<UsageSummary>,
    pub(crate) weekly: Vec<UsageSummary>,
    pub(crate) monthly: Vec<UsageSummary>,
    pub(crate) sessions: Vec<UsageSummary>,
    /// One aggregated row per model, most expensive first.
    pub(crate) models: Vec<UsageSummary>,
}

impl Tables {
    pub(crate) fn is_empty(&self) -> bool {
        self.daily.is_empty() && self.sessions.is_empty()
    }
}

pub(crate) fn load(shared: &SharedArgs) -> Result<Tables> {
    let entries = ccusage_adapter_claude::load_entries(shared, None)?;
    let mut daily = summarize_by_key(
        &entries,
        |entry| entry.date.clone(),
        |key| (key.to_string(), None),
    )?;
    filter_and_sort_summaries(&mut daily, shared, |row| {
        row.date.as_deref().unwrap_or_default()
    });

    let mut weekly = summarize_summaries_by_bucket(&daily, BucketKind::Weekly, WeekDay::Sunday);
    sort_summaries(&mut weekly, &shared.order, |row| {
        row.week.as_deref().unwrap_or_default()
    });

    let mut monthly = summarize_summaries_by_bucket(&daily, BucketKind::Monthly, WeekDay::Sunday);
    sort_summaries(&mut monthly, &shared.order, |row| {
        row.month.as_deref().unwrap_or_default()
    });

    let sessions = session_summaries(&entries, shared)?;
    let models = model_totals(&daily);
    Ok(Tables {
        daily,
        weekly,
        monthly,
        sessions,
        models,
    })
}

/// Aggregates the daily rows' per-model breakdowns into one summary row per
/// model, sorted most expensive first.
pub(crate) fn model_totals(daily: &[UsageSummary]) -> Vec<UsageSummary> {
    let mut order = Vec::new();
    let mut merged = FxHashMap::<String, ModelBreakdown>::default();
    for day in daily {
        for breakdown in &day.model_breakdowns {
            let entry = merged
                .entry(breakdown.model_name.clone())
                .or_insert_with(|| {
                    order.push(breakdown.model_name.clone());
                    ModelBreakdown {
                        model_name: breakdown.model_name.clone(),
                        ..ModelBreakdown::default()
                    }
                });
            entry.input_tokens += breakdown.input_tokens;
            entry.output_tokens += breakdown.output_tokens;
            entry.cache_creation_tokens += breakdown.cache_creation_tokens;
            entry.cache_read_tokens += breakdown.cache_read_tokens;
            entry.extra_total_tokens += breakdown.extra_total_tokens;
            entry.cost += breakdown.cost;
        }
    }
    let mut rows: Vec<UsageSummary> = order
        .into_iter()
        .map(|model| summary_from_breakdown(merged.remove(&model).expect("model was inserted")))
        .collect();
    rows.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));
    rows
}

/// The per-period rows for a single model at the given granularity, oldest
/// first.
pub(crate) fn model_series(
    daily: &[UsageSummary],
    model: &str,
    granularity: Granularity,
) -> Vec<UsageSummary> {
    let mut days: Vec<UsageSummary> = daily
        .iter()
        .filter_map(|day| {
            day.model_breakdowns
                .iter()
                .find(|breakdown| breakdown.model_name == model)
                .map(|breakdown| {
                    let mut row = summary_from_breakdown(breakdown.clone());
                    row.date = day.date.clone();
                    row
                })
        })
        .collect();
    days.sort_by(|a, b| a.date.cmp(&b.date));
    match granularity {
        Granularity::Daily => days,
        Granularity::Weekly => {
            summarize_summaries_by_bucket(&days, BucketKind::Weekly, WeekDay::Sunday)
        }
        Granularity::Monthly => {
            summarize_summaries_by_bucket(&days, BucketKind::Monthly, WeekDay::Sunday)
        }
    }
}

fn summary_from_breakdown(breakdown: ModelBreakdown) -> UsageSummary {
    UsageSummary {
        date: None,
        month: None,
        week: None,
        session_id: None,
        project_path: None,
        last_activity: None,
        first_activity: None,
        input_tokens: breakdown.input_tokens,
        output_tokens: breakdown.output_tokens,
        cache_creation_tokens: breakdown.cache_creation_tokens,
        cache_read_tokens: breakdown.cache_read_tokens,
        extra_total_tokens: breakdown.extra_total_tokens,
        total_cost: breakdown.cost,
        credits: None,
        message_count: None,
        models_used: vec![breakdown.model_name.clone()],
        model_breakdowns: vec![breakdown],
        project: None,
        versions: None,
    }
}

/// Groups entries the same way `ccusage claude session` does, then sorts the
/// rows most-expensive-first, which is the natural reading order for a browser.
fn session_summaries(entries: &[LoadedEntry], shared: &SharedArgs) -> Result<Vec<UsageSummary>> {
    let mut grouped = Vec::<SessionAccumulator>::new();
    let mut group_indexes = FxHashMap::<(Arc<str>, Arc<str>), usize>::default();
    for entry in entries {
        let key = (
            Arc::clone(&entry.project_path),
            Arc::clone(&entry.session_id),
        );
        let index = *group_indexes.entry(key).or_insert_with(|| {
            grouped.push(SessionAccumulator::default());
            grouped.len() - 1
        });
        grouped[index].add_entry(entry);
    }

    let mut rows = Vec::with_capacity(grouped.len());
    for group in grouped {
        rows.push(group.into_summary()?);
    }
    if shared.since.is_some() || shared.until.is_some() {
        rows.retain(|row| {
            let date = row
                .last_activity
                .as_deref()
                .unwrap_or_default()
                .replace('-', "");
            shared.since.as_ref().is_none_or(|since| &date >= since)
                && shared.until.as_ref().is_none_or(|until| &date <= until)
        });
    }
    rows.retain(|row| {
        row.input_tokens + row.output_tokens + row.cache_creation_tokens + row.cache_read_tokens > 0
    });
    rows.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));
    Ok(rows)
}

#[derive(Default)]
pub(crate) struct Totals {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cache_creation_tokens: u64,
    pub(crate) cache_read_tokens: u64,
    pub(crate) total_tokens: u64,
    pub(crate) cost: f64,
}

pub(crate) fn totals(rows: &[UsageSummary]) -> Totals {
    let mut totals = Totals::default();
    for row in rows {
        totals.input_tokens += row.input_tokens;
        totals.output_tokens += row.output_tokens;
        totals.cache_creation_tokens += row.cache_creation_tokens;
        totals.cache_read_tokens += row.cache_read_tokens;
        totals.total_tokens += row.total_tokens();
        totals.cost += row.total_cost;
    }
    totals
}

/// Abbreviates a token count to three significant digits with a k/M/B
/// suffix, e.g. `31.1k`, `1.76M`, `340M`. The tables keep full counts; this
/// is only for the totals footer.
pub(crate) fn format_compact_number(value: u64) -> String {
    const UNITS: [(f64, &str); 3] = [(1e9, "B"), (1e6, "M"), (1e3, "k")];
    for (divisor, suffix) in UNITS {
        let scaled = value as f64 / divisor;
        if scaled >= 1.0 {
            return if scaled >= 100.0 {
                format!("{scaled:.0}{suffix}")
            } else if scaled >= 10.0 {
                format!("{scaled:.1}{suffix}")
            } else {
                format!("{scaled:.2}{suffix}")
            };
        }
    }
    value.to_string()
}

#[cfg(test)]
pub(crate) mod fixtures {
    use ccusage_core::ModelBreakdown;

    use super::*;

    pub(crate) struct RowFixture {
        pub(crate) date: Option<&'static str>,
        pub(crate) session: Option<(&'static str, &'static str)>,
        pub(crate) model: &'static str,
        pub(crate) input_tokens: u64,
        pub(crate) cost: f64,
    }

    pub(crate) fn row(fixture: RowFixture) -> UsageSummary {
        UsageSummary {
            date: fixture.date.map(str::to_string),
            month: None,
            week: None,
            session_id: fixture.session.map(|(_, session)| session.to_string()),
            project_path: fixture.session.map(|(project, _)| project.to_string()),
            last_activity: fixture.date.map(str::to_string),
            first_activity: None,
            input_tokens: fixture.input_tokens,
            output_tokens: 10,
            cache_creation_tokens: 1,
            cache_read_tokens: 2,
            extra_total_tokens: 0,
            total_cost: fixture.cost,
            credits: None,
            message_count: None,
            models_used: vec![fixture.model.to_string()],
            model_breakdowns: vec![ModelBreakdown {
                model_name: fixture.model.to_string(),
                input_tokens: fixture.input_tokens,
                output_tokens: 10,
                cache_creation_tokens: 1,
                cache_read_tokens: 2,
                extra_total_tokens: 0,
                cost: fixture.cost,
                missing_pricing: false,
            }],
            project: None,
            versions: None,
        }
    }

    pub(crate) fn tables() -> Tables {
        let daily = vec![
            row(RowFixture {
                date: Some("2026-07-01"),
                session: None,
                model: "claude-sonnet-5",
                input_tokens: 100,
                cost: 1.0,
            }),
            row(RowFixture {
                date: Some("2026-07-02"),
                session: None,
                model: "claude-fable-5",
                input_tokens: 200,
                cost: 2.0,
            }),
            row(RowFixture {
                date: Some("2026-07-03"),
                session: None,
                model: "claude-sonnet-5",
                input_tokens: 300,
                cost: 3.0,
            }),
        ];
        let models = model_totals(&daily);
        Tables {
            daily,
            weekly: vec![{
                let mut weekly = row(RowFixture {
                    date: None,
                    session: None,
                    model: "claude-sonnet-5",
                    input_tokens: 600,
                    cost: 6.0,
                });
                weekly.week = Some("2026-06-28".to_string());
                weekly
            }],
            monthly: vec![{
                let mut monthly = row(RowFixture {
                    date: None,
                    session: None,
                    model: "claude-sonnet-5",
                    input_tokens: 600,
                    cost: 6.0,
                });
                monthly.month = Some("2026-07".to_string());
                monthly
            }],
            sessions: vec![
                row(RowFixture {
                    date: Some("2026-07-03"),
                    session: Some(("/home/user/project-a", "session-a")),
                    model: "claude-sonnet-5",
                    input_tokens: 400,
                    cost: 4.0,
                }),
                row(RowFixture {
                    date: Some("2026-07-02"),
                    session: Some(("/home/user/project-b", "session-b")),
                    model: "claude-fable-5",
                    input_tokens: 200,
                    cost: 2.0,
                }),
            ],
            models,
        }
    }

    pub(crate) fn empty_tables() -> Tables {
        Tables {
            daily: Vec::new(),
            weekly: Vec::new(),
            monthly: Vec::new(),
            sessions: Vec::new(),
            models: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures::tables, *};

    #[test]
    fn model_totals_aggregates_across_days_most_expensive_first() {
        let models = model_totals(&tables().daily);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].models_used, vec!["claude-sonnet-5"]);
        assert_eq!(models[0].total_cost, 4.0);
        assert_eq!(models[0].input_tokens, 400);
        assert_eq!(models[1].models_used, vec!["claude-fable-5"]);
        assert_eq!(models[1].total_cost, 2.0);
    }

    #[test]
    fn model_series_filters_to_one_model_per_day() {
        let series = model_series(&tables().daily, "claude-sonnet-5", Granularity::Daily);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].date.as_deref(), Some("2026-07-01"));
        assert_eq!(series[1].date.as_deref(), Some("2026-07-03"));
        assert_eq!(series[1].total_cost, 3.0);
    }

    #[test]
    fn model_series_rebuckets_to_monthly() {
        let series = model_series(&tables().daily, "claude-sonnet-5", Granularity::Monthly);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].month.as_deref(), Some("2026-07"));
        assert_eq!(series[0].total_cost, 4.0);
        assert_eq!(series[0].input_tokens, 400);
    }

    #[test]
    fn model_series_is_chronological_even_when_daily_rows_are_not() {
        let mut daily = tables().daily;
        daily.reverse();
        let series = model_series(&daily, "claude-sonnet-5", Granularity::Daily);
        assert_eq!(series[0].date.as_deref(), Some("2026-07-01"));
    }

    #[test]
    fn compact_numbers_use_three_significant_digits() {
        assert_eq!(format_compact_number(0), "0");
        assert_eq!(format_compact_number(999), "999");
        assert_eq!(format_compact_number(1_000), "1.00k");
        assert_eq!(format_compact_number(31_100), "31.1k");
        assert_eq!(format_compact_number(1_760_000), "1.76M");
        assert_eq!(format_compact_number(3_970_000), "3.97M");
        assert_eq!(format_compact_number(340_000_000), "340M");
        assert_eq!(format_compact_number(1_500_000_000), "1.50B");
    }
}
