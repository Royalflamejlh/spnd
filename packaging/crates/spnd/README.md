# spnd

An interactive terminal UI and reports for coding-agent token usage and spend —
a fork of [ccusage](https://github.com/ccusage/ccusage) with a full ratatui
browser (`spnd` opens it directly).

This crates.io package is a pointer: the real application is a Cargo workspace
with internal path dependencies, so install it with one of:

```bash
cargo install --git https://github.com/Royalflamejlh/spnd spnd
npm install -g @spnd/spnd
curl -fsSL https://raw.githubusercontent.com/Royalflamejlh/spnd/main/install.sh | sh
```

Prebuilt binaries for Linux, macOS, and Windows are on the
[releases page](https://github.com/Royalflamejlh/spnd/releases).
