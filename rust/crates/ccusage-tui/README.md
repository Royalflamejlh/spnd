# ccusage-tui

The interactive `ccusage tui` terminal UI: a ratatui application over the Claude
Code usage data, with daily, monthly, and session report tabs, keyboard
navigation, and a per-row model breakdown view.

## Owns

- `lib.rs` — `run`, the terminal lifecycle, and the event loop.
- `data.rs` — loading entries through the Claude adapter and shaping the daily,
  monthly, and session rows the tabs display.
- `app.rs` — the application state: active tab, per-tab selection, sort
  direction, and the breakdown popup flag.
- `input.rs` — key handling.
- `ui.rs` — rendering: the tab bar, report tables, totals footer, and the model
  breakdown popup.

## Public surface

- `run`

## Depends on

- `ccusage-adapter-claude`
- `ccusage-core`
- `ratatui`

## Build layer

Built in the final binary Crane artifact layer alongside the `ccusage` binary,
so a change here only recompiles that last layer.
