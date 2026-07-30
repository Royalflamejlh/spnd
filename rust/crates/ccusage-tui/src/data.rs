//! Shapes loaded usage entries into the rows each tab displays.
use std::sync::Arc;

use ccusage_core::{
    BucketKind, LoadedEntry, Result, SessionAccumulator, UsageSummary,
    cli::{SharedArgs, WeekDay},
    fast::FxHashMap,
    filter_and_sort_summaries, sort_summaries, summarize_by_key, summarize_summaries_by_bucket,
};

pub(crate) struct Tables {
    pub(crate) daily: Vec<UsageSummary>,
    pub(crate) weekly: Vec<UsageSummary>,
    pub(crate) monthly: Vec<UsageSummary>,
    pub(crate) sessions: Vec<UsageSummary>,
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
    Ok(Tables {
        daily,
        weekly,
        monthly,
        sessions,
    })
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
        Tables {
            daily: vec![
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
            ],
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
        }
    }

    pub(crate) fn empty_tables() -> Tables {
        Tables {
            daily: Vec::new(),
            weekly: Vec::new(),
            monthly: Vec::new(),
            sessions: Vec::new(),
        }
    }
}
