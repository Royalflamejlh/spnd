# Contributing to spnd

Issues, bug reports, and pull requests are all welcome.

`spnd` is a fork of [ccusage](https://github.com/ccusage/ccusage) that adds an
interactive terminal UI. If your report is about a report command rather than
the TUI, it may also affect upstream — worth checking there too.

## The One Rule

**You must understand your change.** If you cannot explain what your code does
and how it interacts with the rest of the project, it is not ready for review.

Using AI tools is fine. Submitting generated output that you have not read,
run, or tested is not. If you use an agent, run it from the repository root so
it picks up `CLAUDE.md` and the repo-local skills.

## Filing an Issue

Use a template if one fits, or open a blank issue if none do. Say what happened,
what you expected, and which version you are on (`spnd --version`). For anything
larger than a small fix, an issue first is usually faster than a surprise PR.

## Development Setup

This repository uses a pinned Nix dev shell with direnv, which puts the right
Rust toolchain, pnpm, git hooks, and repo CLIs on `PATH`:

```bash
direnv allow
just install
```

`just install` is only needed once per checkout, and after a lockfile change.
`git wt` runs it for you when it creates a worktree.

## Before Submitting a PR

```bash
just fmt
just typecheck
just test
```

`just --list` shows everything else; `just check` runs the full flake check
suite (treefmt, oxlint, clippy, schema drift, gitleaks) the way CI does.

A few conventions worth knowing:

- The canonical command is `spnd`, with agent subcommands like `spnd codex` and
  `spnd opencode`. Standalone wrapper packages (`ccusage-codex`,
  `ccusage-opencode`, `ccusage-amp`, `ccusage-pi`) were removed and should not
  be reintroduced.
- The production CLI is Rust-first under `rust/crates` and `rust/adapters`.
- Don't add documentation files unless the change needs user-facing docs.

## FAQ

### Is AI-generated code banned?

No. The requirement is that you understand the change, have tested it, and can
explain it in your own words.

### Why is CI running so many jobs on my PR?

Every PR builds native binaries for five platforms. Add the `perf` label if you
also want a pkg.pr.new preview package and a before/after benchmark comment.
