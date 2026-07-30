# Interactive TUI

The `ccusage tui` command opens an interactive terminal UI for browsing Claude
Code usage instead of printing a one-shot report. It loads the same data as the
`claude` reports and lets you flip between daily, monthly, and session views,
scroll rows, toggle sort direction, and inspect the per-model breakdown of any
row.

## Basic Usage

```bash
ccusage tui
```

## Example Output

<!-- eslint-skip -->

```
 ccusage   Daily │ Monthly │ Sessions                                    [asc]
┌ Daily — 3 rows ─────────────────────────────────────────────────────────────┐
│Date            Input     Output  Cache Create   Cache Read  Total  Cost ... │
│2026-05-14      1,887    183,055           128          512  185,582  $81.73 │
│2026-05-15      2,775    186,645           256          768  190,444  $98.45 │
│2026-05-16      4,512    285,846           512        1,024  291,894 $156.40 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
 Totals  In 9,174  Out 655,546  Cache 896/2,304  Tokens 667,920  $336.58
 q quit · tab/←→ switch · ↑↓/jk move · g/G ends · s sort · enter breakdown
```

## Views

- **Daily**: usage grouped by date, matching `ccusage claude daily`.
- **Monthly**: daily rows bucketed by month, matching `ccusage claude monthly`.
- **Sessions**: usage grouped by project and session, sorted
  most-expensive-first, matching `ccusage claude session`.

The totals line always reflects the rows in the current view, and selecting a
row and pressing <kbd>Enter</kbd> opens its per-model cost breakdown.

## Keyboard Controls

| Key                                             | Action                                    |
| ----------------------------------------------- | ----------------------------------------- |
| <kbd>q</kbd> / <kbd>Esc</kbd>                   | Quit (or close the breakdown popup)       |
| <kbd>Tab</kbd> / <kbd>←</kbd> <kbd>→</kbd>      | Switch between Daily, Monthly, Sessions   |
| <kbd>↑</kbd> <kbd>↓</kbd> / <kbd>j</kbd> <kbd>k</kbd> | Move the row selection               |
| <kbd>PgUp</kbd> / <kbd>PgDn</kbd>               | Move the selection ten rows               |
| <kbd>g</kbd> / <kbd>G</kbd>                     | Jump to the first / last row              |
| <kbd>s</kbd>                                    | Toggle sort direction for the current tab |
| <kbd>Enter</kbd> / <kbd>b</kbd>                 | Open the model breakdown for the row      |

## Command Options

`ccusage tui` accepts the shared Claude report options, so the data it browses
can be filtered and shaped the same way as the printed reports:

```bash
# Only browse a date range
ccusage tui --since 20260101 --until 20260131

# Force cost calculation from tokens and use a fixed timezone
ccusage tui --mode calculate --timezone UTC

# Use cached pricing without network access
ccusage tui --offline
```

Run `ccusage tui --help` for the full list. Options that only affect printed
output, such as `--json`, are accepted but have no effect inside the TUI.

## Related Guides

- [Daily Usage](/guide/daily-reports) — the printed daily report
- [Monthly Usage](/guide/monthly-reports) — the printed monthly report
- [Session Usage](/guide/session-reports) — the printed session report
- [Command-Line Options](/guide/cli-options) — shared option reference
