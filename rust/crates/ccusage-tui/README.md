# ccusage-tui

The interactive `spnd tui` terminal UI: a ratatui application over the Claude
Code usage data, with Usage/Sessions/Models pages, daily/weekly/monthly
bucketing, a cost bar chart, per-model drill-down pages, column sorting, and
full mouse support.

## Owns

- `lib.rs` — `run`, the terminal lifecycle (including mouse capture with a
  panic-safe restore chain), and the event loop.
- `data.rs` — loading entries through the Claude adapter and shaping the
  daily/weekly/monthly, session, and per-model rows the pages display.
- `app.rs` — the application state: active tab, granularity, per-view
  selection and sort, the model drill-down, and the popup flags.
- `action.rs` — the action vocabulary every input source maps into, and the
  `update` function that applies one action to the state.
- `input.rs` — key and mouse event translation into actions.
- `hit.rs` — the per-frame hit map from screen regions to actions, which makes
  mouse clicks dispatch exactly what the keyboard would.
- `ui.rs` — rendering: the tab bar, `[D][W][M]` control, chart strip, report
  tables with sortable headers, scrollbar, totals footer, model breakdown
  popup, and help overlay.

## Public surface

- `run`

## Depends on

- `ccusage-adapter-claude`
- `ccusage-core`
- `ratatui`

## Build layer

Built in the final binary Crane artifact layer alongside the `spnd` binary,
so a change here only recompiles that last layer.
