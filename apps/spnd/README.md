# spnd

[![GitHub release](https://img.shields.io/github/v/release/Royalflamejlh/spnd?color=blue)](https://github.com/Royalflamejlh/spnd/releases)
[![crates.io](https://img.shields.io/crates/v/spnd?color=orange)](https://crates.io/crates/spnd)
[![npm](https://img.shields.io/npm/v/%40spnd%2Fspnd?color=yellow)](https://www.npmjs.com/package/@spnd/spnd)
[![CI](https://github.com/Royalflamejlh/spnd/actions/workflows/ci.yaml/badge.svg)](https://github.com/Royalflamejlh/spnd/actions/workflows/ci.yaml)
[![license: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

An interactive terminal UI and reports for coding-agent token usage and spend.
Running `spnd` opens the TUI directly. `spnd` is a fork of
[ccusage/ccusage](https://github.com/ccusage/ccusage) — everything besides the
TUI works exactly like upstream; see the
[upstream repository](https://github.com/ccusage/ccusage) and
[ccusage.com](https://ccusage.com/) for full documentation of the report
commands, supported sources, and options (substitute `spnd` for `ccusage`).

## Installing

| Method                                                                              | Command                                                                                                                                                 |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Install script (Linux/macOS)                                                        | `curl -fsSL https://raw.githubusercontent.com/Royalflamejlh/spnd/main/install.sh \| sh`                                                                 |
| [Homebrew](https://github.com/Royalflamejlh/spnd/blob/main/Formula/spnd.rb)         | `brew tap royalflamejlh/spnd https://github.com/Royalflamejlh/spnd && brew install spnd`                                                                |
| [Scoop](https://github.com/Royalflamejlh/spnd/blob/main/bucket/spnd.json) (Windows) | `scoop bucket add spnd https://github.com/Royalflamejlh/spnd && scoop install spnd`                                                                     |
| [npm](https://www.npmjs.com/package/@spnd/spnd)                                     | `npm install -g @spnd/spnd`                                                                                                                             |
| [Cargo](https://crates.io/crates/spnd)                                              | `cargo install --git https://github.com/Royalflamejlh/spnd spnd`                                                                                        |
| Nix                                                                                 | `nix run github:Royalflamejlh/spnd`                                                                                                                     |
| deb/rpm                                                                             | download from [releases](https://github.com/Royalflamejlh/spnd/releases), then `apt install ./spnd-linux-x64.deb` or `dnf install ./spnd-linux-x64.rpm` |
| Prebuilt binaries                                                                   | tarballs and zips for Linux, macOS, and Windows on the [releases page](https://github.com/Royalflamejlh/spnd/releases)                                  |

The crates.io `spnd` package is a pointer crate (the workspace's internal path
dependencies cannot be published); `cargo install` from git builds the real
binary.

## Interactive TUI

```bash
spnd                              # opens the TUI
spnd --since 20260101 --offline   # same, filtered
```

- **Usage / Sessions / Models pages** over the same data as the `claude` reports
- **D/W/M bucketing**: switch daily, weekly, or monthly with the `[D][W][M]` control or the `d`/`w`/`m` keys
- **Cost chart**: a clickable bar chart above bucketed tables; bars select their row
- **Model drill-down**: open a model from the Models page (or from any breakdown popup) to see its usage over time; `[`/`]` page between models
- **Column sorting**: click any column header, or use `s` (flip direction) and `o` (next column)
- **Full mouse support**: click tabs/rows/headers/bars, scroll with the wheel, drag the scrollbar, right-click to back out
- **Help overlay** on `?`, totals footer that always reflects the current view

Piped output and `--json` skip the TUI and print the unified daily report, so
scripts keep working; the report commands (`spnd daily`, `spnd monthly`,
`spnd session`, per-agent subcommands, and so on) are unchanged.

## Building from Source

```bash
cd rust
cargo build --release                 # binary at rust/target/release/spnd
cargo install --path crates/ccusage   # install spnd to ~/.cargo/bin
```

## License

[MIT](LICENSE) © [@ryoppippi](https://github.com/ryoppippi)
