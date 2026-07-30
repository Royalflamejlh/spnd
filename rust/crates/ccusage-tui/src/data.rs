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
    pub(crate) monthly: Vec<UsageSummary>,
    pub(crate) sessions: Vec<UsageSummary>,
}

impl Tables {
    pub(crate) fn is_empty(&self) -> bool {
        self.daily.is_empty() && self.monthly.is_empty() && self.sessions.is_empty()
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

    let mut monthly = summarize_summaries_by_bucket(&daily, BucketKind::Monthly, WeekDay::Sunday);
    sort_summaries(&mut monthly, &shared.order, |row| {
        row.month.as_deref().unwrap_or_default()
    });

    let sessions = session_summaries(&entries, shared)?;
    Ok(Tables {
        daily,
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
