# All Sources (Default)

![spnd daily report showing unified token usage and costs across sources](/screenshot.png)

spnd aggregates every detected supported data source by default. You do not need a special `all` command or flag for the common case.

## Basic Usage

```bash
# Daily usage across every detected source
spnd
spnd daily

# Other unified views
spnd weekly
spnd monthly
spnd session
```

The `--all` flag is accepted for compatibility, but it is optional because unified views are already the default.

```bash
spnd daily --all
```

For automation, unified JSON reports can emit several report sections from one load:

```bash
spnd daily --sections daily,monthly,session --json
spnd daily --by-agent --json
```

`--sections` accepts `daily`, `weekly`, `monthly`, and `session`. The invoked report section is always included, and table output prints each requested section as a separate table. `--by-agent` adds an `agents` array to daily, weekly, and monthly JSON rows; session rows are already source-specific.

## How Unified Views Work

spnd detects local usage files from Claude Code, Codex, OpenCode, Amp, Droid, Codebuff, Hermes Agent, pi-agent, Goose, OpenClaw, Kilo, Kimi, Qwen, GitHub Copilot CLI, and Gemini CLI. The same daily, weekly, monthly, and session views can run in two modes:

| Mode    | Command example     | What it shows                           |
| ------- | ------------------- | --------------------------------------- |
| Unified | `spnd daily`        | Every detected supported source         |
| Focused | `spnd codex daily`  | One source using the same report shape  |
| Focused | `spnd claude daily` | One source with source-specific options |

Unified tables include an **Agent** column so you can compare sources in one view. Focused views remove that comparison layer and show the selected source in more detail where applicable.

## Supported Sources

| Source       | Namespace  | Example focused view   |
| ------------ | ---------- | ---------------------- |
| Claude Code  | `claude`   | `spnd claude daily`    |
| Codex        | `codex`    | `spnd codex daily`     |
| OpenCode     | `opencode` | `spnd opencode weekly` |
| Amp          | `amp`      | `spnd amp session`     |
| Droid        | `droid`    | `spnd droid daily`     |
| Codebuff     | `codebuff` | `spnd codebuff daily`  |
| Hermes Agent | `hermes`   | `spnd hermes daily`    |
| pi-agent     | `pi`       | `spnd pi monthly`      |
| Goose        | `goose`    | `spnd goose daily`     |
| OpenClaw     | `openclaw` | `spnd openclaw daily`  |
| Kilo         | `kilo`     | `spnd kilo daily`      |
| Kimi         | `kimi`     | `spnd kimi daily`      |
| Qwen         | `qwen`     | `spnd qwen daily`      |
| Copilot CLI  | `copilot`  | `spnd copilot daily`   |
| Gemini CLI   | `gemini`   | `spnd gemini daily`    |

## When to Focus a Source

Use a source namespace when you want source-specific options or when you are debugging one local data format:

```bash
spnd codex daily --speed fast
spnd claude daily --mode display
spnd opencode session --json
spnd amp monthly --compact
spnd droid session
spnd codebuff daily
spnd pi session --pi-path /path/to/sessions
spnd openclaw daily --open-claw-path /path/to/openclaw
spnd kilo session
spnd qwen daily
spnd copilot daily --json
spnd gemini session --json
```

## Next Steps

- [Daily Usage](/guide/daily-reports) - Calendar-day usage
- [Weekly Usage](/guide/weekly-reports) - Week-by-week usage
- [Monthly Usage](/guide/monthly-reports) - Longer-term usage trends
- [Session Usage](/guide/session-reports) - Per-conversation usage
- [Data Sources](/guide/#data-sources) - Supported local data formats
- [Source Support Q&A](/guide/source-support-qa) - Why some investigated CLIs are not supported
