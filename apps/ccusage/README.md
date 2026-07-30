# ccusageui

A fork of [ccusage/ccusage](https://github.com/ccusage/ccusage) that adds `ccusage tui`, an
interactive [ratatui](https://ratatui.rs/) terminal UI for browsing Claude Code usage with
daily, monthly, and session tabs. Everything else works exactly like upstream ccusage — see
the [upstream repository](https://github.com/ccusage/ccusage) and
[ccusage.com](https://ccusage.com/) for full documentation of the CLI, supported sources,
and report options.

## Interactive TUI

```bash
ccusage tui
ccusage tui --since 20260101 --offline
```

- **Usage / Sessions / Models pages** over the same data as the `claude` reports
- **D/W/M bucketing**: switch daily, weekly, or monthly with the `[D][W][M]` control or the `d`/`w`/`m` keys
- **Cost chart**: a clickable bar chart above bucketed tables; bars select their row
- **Model drill-down**: open a model from the Models page (or from any breakdown popup) to see its usage over time; `[`/`]` page between models
- **Column sorting**: click any column header, or use `s` (flip direction) and `o` (next column)
- **Full mouse support**: click tabs/rows/headers/bars, scroll with the wheel, drag the scrollbar, right-click to back out
- **Help overlay** on `?`, totals footer that always reflects the current view
- Accepts the shared Claude report options such as `--since`, `--until`, `--timezone`, `--mode`, and `--offline`

The TUI ships in this fork's `ccusage` binary only; the upstream npm package does not include it.

## Building and Installing

Build the Rust CLI from this repository:

```bash
cd rust
cargo build --release                 # binary at rust/target/release/ccusage
cargo install --path crates/ccusage   # install to ~/.cargo/bin
```

Or run it via Nix:

```bash
nix run github:Royalflamejlh/ccusageui -- tui
```

All of the regular ccusage commands (`ccusage daily`, `ccusage monthly`, `ccusage session`,
per-agent subcommands, and so on) work from the same binary.

## License

[MIT](LICENSE) © [@ryoppippi](https://github.com/ryoppippi)
