# Interactive TUI

The `spnd tui` command opens an interactive terminal UI for browsing Claude
Code usage instead of printing a one-shot report. It loads the same data as the
`claude` reports and lets you flip between usage, session, and model views,
switch between daily, weekly, and monthly bucketing, sort by any column, and
drill into any model's usage over time — with the keyboard or the mouse.

## Basic Usage

```bash
spnd tui
```

## Example Output

<!-- eslint-skip -->

```
 spnd   Usage │ Sessions │ Models                        [D][W][M]  [key▴]
┌ cost per daily ─────────────────────────────────────────────────────────────┐
│                                        ████                                 │
│                          ████    ████  ████                                 │
│              ████  ████  ████    ████  ████                                 │
│  14    15    16    17    18      19    20                                   │
└─────────────────────────────────────────────────────────────────────────────┘
┌ Daily — 7 rows ─────────────────────────────────────────────────────────────┐
│Date ▴          Input     Output  Cache Create   Cache Read  Total  Cost ... │
│2026-05-14      1,887    183,055           128          512  185,582  $81.73 │
│2026-05-15      2,775    186,645           256          768  190,444  $98.45 │
│2026-05-16      4,512    285,846           512        1,024  291,894 $156.40 │
└─────────────────────────────────────────────────────────────────────────────┘
 Totals  In 9,174  Out 655,546  Cache 896/2,304  Tokens 667,920  $336.58
 q quit · tab switch · d/w/m bucket · ↑↓ move · s/o sort · enter breakdown · ? help
```

## Pages

- **Usage**: usage bucketed by day, week, or month — switch with the
  `[D][W][M]` control or the <kbd>d</kbd>/<kbd>w</kbd>/<kbd>m</kbd> keys. A
  cost bar chart sits above the table; clicking a bar selects its row.
- **Sessions**: usage grouped by project and session, sorted
  most-expensive-first, matching `spnd claude session`.
- **Models**: one row per model with aggregated tokens, cost, and a
  share-of-total-cost bar. Opening a row (Enter, or clicking the selected row)
  drills into that model's usage over time at the current bucketing;
  <kbd>[</kbd> and <kbd>]</kbd> page between models without backing out.

The totals line always reflects the rows in the current view. Selecting a row
and pressing <kbd>Enter</kbd> opens its per-model cost breakdown, and clicking
a model inside that popup jumps straight to the model's drill-down page.

## Keyboard Controls

| Key                                                   | Action                                        |
| ----------------------------------------------------- | --------------------------------------------- |
| <kbd>q</kbd> / <kbd>Ctrl-C</kbd>                      | Quit                                          |
| <kbd>Esc</kbd> / <kbd>Backspace</kbd>                 | Close the popup / leave the drill-down / quit |
| <kbd>Tab</kbd> / <kbd>←</kbd> <kbd>→</kbd>            | Switch between Usage, Sessions, Models        |
| <kbd>d</kbd> / <kbd>w</kbd> / <kbd>m</kbd>            | Daily, weekly, or monthly bucketing           |
| <kbd>↑</kbd> <kbd>↓</kbd> / <kbd>j</kbd> <kbd>k</kbd> | Move the row selection                        |
| <kbd>PgUp</kbd> / <kbd>PgDn</kbd>                     | Move the selection ten rows                   |
| <kbd>g</kbd> / <kbd>G</kbd>                           | Jump to the first / last row                  |
| <kbd>s</kbd>                                          | Flip the sort direction                       |
| <kbd>o</kbd>                                          | Sort by the next column                       |
| <kbd>Enter</kbd> / <kbd>b</kbd>                       | Model breakdown (Models page: drill in)       |
| <kbd>[</kbd> / <kbd>]</kbd>                           | Previous / next model in the drill-down       |
| <kbd>?</kbd>                                          | Key and mouse reference overlay               |

## Mouse Controls

The TUI captures the mouse, so everything above is also clickable:

- Click a tab, a `[D]`/`[W]`/`[M]` segment, or a column header (headers sort,
  clicking again flips the direction).
- Click a row to select it; click the selected row again to open its
  breakdown, or to drill in on the Models page.
- Click a bar in the chart to select the matching table row.
- Scroll with the wheel; long tables get a scrollbar you can click or drag.
- Right-click backs out of popups and drill-downs.

Terminals usually reserve <kbd>Shift</kbd>+drag for native text selection
while a program captures the mouse.

## Command Options

`spnd tui` accepts the shared Claude report options, so the data it browses
can be filtered and shaped the same way as the printed reports:

```bash
# Only browse a date range
spnd tui --since 20260101 --until 20260131

# Force cost calculation from tokens and use a fixed timezone
spnd tui --mode calculate --timezone UTC

# Use cached pricing without network access
spnd tui --offline
```

Run `spnd tui --help` for the full list. Options that only affect printed
output, such as `--json`, are accepted but have no effect inside the TUI.

## Related Guides

- [Daily Usage](/guide/daily-reports) — the printed daily report
- [Monthly Usage](/guide/monthly-reports) — the printed monthly report
- [Session Usage](/guide/session-reports) — the printed session report
- [Command-Line Options](/guide/cli-options) — shared option reference
