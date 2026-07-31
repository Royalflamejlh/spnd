# Command-Line Options

spnd provides extensive command-line options to customize its behavior. These options take precedence over configuration files and environment variables.

## Global Options

All spnd commands support these global options:

### Date Filtering

Filter usage data by date range:

```bash
# Filter by date range
spnd daily --since 20260101 --until 20260531

# Show data from a specific date
spnd monthly --since 20260101

# Show data up to a specific date
spnd session --until 20260531
```

### Recent Periods

Instead of working out dates, ask for the most recent periods of whatever the report groups by:

```bash
# Today
spnd daily --last 1

# This week
spnd weekly --last 1

# This month
spnd monthly --last 1

# The last seven days, and the last three months
spnd daily --last 7
spnd monthly --last 3
```

The count is inclusive of the current period, so `--last 2` on a daily report covers yesterday and today. Weeks start on the same day the report buckets by, which is Monday everywhere except `spnd claude weekly`, where `--start-of-week` decides.

`--last` works on every daily, weekly, and monthly report, including the per-agent ones such as `spnd codex daily --last 1`. It is not available on `session`, `blocks`, `statusline`, or `tui`, which have no calendar period, and it cannot be combined with `--since`, `--until`, or `--sections`.

### Output Format

Control how data is displayed:

```bash
# JSON output for programmatic use
spnd daily --json
spnd daily -j

# Show per-model breakdown
spnd daily --breakdown
spnd daily -b

# Hide cost columns and JSON cost fields
spnd daily --no-cost
spnd daily --json --no-cost

# Combine options
spnd daily --json --breakdown
```

`--no-cost` removes cost columns from table output and removes cost fields such as `totalCost`, `costUSD`, and `cost` from JSON output.

### Cost Calculation Mode

Choose how costs are calculated:

```bash
# Auto mode (default) - use costUSD when available
spnd daily --mode auto

# Calculate mode - always calculate from tokens
spnd daily --mode calculate

# Display mode - only show pre-calculated costUSD
spnd daily --mode display
```

### Sort Order

Control the ordering of results:

```bash
# Newest first (default)
spnd daily --order desc

# Oldest first
spnd daily --order asc
```

### Offline Mode

Run without network connectivity:

```bash
# Use cached pricing data
spnd daily --offline
spnd daily -O
```

### Timezone

Set the timezone for date calculations:

```bash
# Use UTC timezone
spnd daily --timezone UTC

# Use specific timezone
spnd daily --timezone America/New_York
spnd daily -z Asia/Tokyo

# Short alias
spnd monthly -z Europe/London
```

#### Timezone Effect

The timezone affects how usage is grouped by date. For example, usage at 11 PM UTC on January 1st would appear on:

- **January 1st** when `--timezone UTC`
- **January 1st** when `--timezone America/New_York` (6 PM EST)
- **January 2nd** when `--timezone Asia/Tokyo` (8 AM JST next day)

### Debug Options

Get detailed debugging information:

```bash
# Debug mode - show pricing mismatches and config loading
spnd daily --debug

# Show sample discrepancies
spnd daily --debug --debug-samples 10
```

### Configuration File

Use a custom configuration file:

```bash
# Specify custom config file
spnd daily --config ./my-config.json
spnd monthly --config /path/to/team-config.json
```

## Command-Specific Options

### Unified Report Options

These options apply to `spnd daily`, `spnd weekly`, `spnd monthly`, and `spnd session` when they are aggregating all detected sources:

```bash
# Emit several JSON report sections from one source load
spnd daily --sections daily,monthly,session --json

# Add per-agent breakdowns to daily, weekly, and monthly JSON rows
spnd daily --by-agent --json
```

`--sections` accepts a comma-separated list of `daily`, `weekly`, `monthly`, and `session`. The invoked report section is always included. For table output, each requested section is printed as a separate table. `--by-agent` is JSON-only; session rows are already per-agent.

### Daily Command

Additional options for daily reports:

```bash
# Group by project
spnd daily --instances
spnd daily -i

# Filter to specific project
spnd daily --project myproject
spnd daily -p myproject

# Combine project filtering
spnd daily --instances --project myproject
```

### Weekly Command

Options for weekly reports:

```bash
# Set week start day
spnd weekly --start-of-week monday
spnd weekly --start-of-week sunday
```

### Session Command

Options for session reports:

```bash
# Filter by session ID
spnd session --id abc123-session

# Filter by project
spnd session --project myproject
```

### Blocks Command

Options for 5-hour billing blocks:

```bash
# Show only active block
spnd blocks --active
spnd blocks -a

# Show recent blocks (last 3 days)
spnd blocks --recent
spnd blocks -r

# Set token limit for warnings
spnd blocks --token-limit 500000
spnd blocks --token-limit max

# Live monitoring mode
spnd blocks --live
spnd blocks --live --refresh-interval 2

# Customize session length
spnd blocks --session-length 5
```

### Statusline

Options for statusline display:

```bash
# Basic statusline
spnd statusline

# Force offline mode
spnd statusline --offline

# Enable caching
spnd statusline --cache

# Custom refresh interval
spnd statusline --refresh-interval 5
```

## JSON Output

```bash
# Print JSON output
spnd daily --json

# Print JSON without cost fields
spnd daily --json --no-cost

# Pipe JSON output to jq
spnd daily --json | jq ".data[]"

# Extract specific fields
spnd session --json | jq ".data[] | {date, cost}"
```

## Option Precedence

Options are applied in this order (highest to lowest priority):

1. **Command-line arguments** - Direct CLI options
2. **Custom config file** - Via `--config` flag
3. **Local project config** - `.spnd/spnd.json`
4. **User config** - `~/.config/claude/spnd.json`
5. **Legacy config** - `~/.claude/spnd.json`
6. **Built-in defaults**

## Examples

### Development Workflow

```bash
# Daily development check
spnd daily --instances --breakdown

# Check specific project costs
spnd daily --project myapp --since 20260101

# Export for reporting
spnd monthly --json > monthly-report.json
```

### Team Collaboration

```bash
# Use team configuration
spnd daily --config ./team-config.json

# Consistent timezone for remote team
spnd daily --timezone UTC

# Generate shareable report
spnd weekly --json
```

### Cost Monitoring

```bash
# Monitor active usage
spnd blocks --active --live

# Check if approaching limits
spnd blocks --token-limit 500000

# Historical analysis
spnd monthly --mode calculate --breakdown
```

### Debugging Issues

```bash
# Debug configuration loading
spnd daily --debug --config ./test-config.json

# Check pricing discrepancies
spnd daily --debug --debug-samples 20

# Silent mode for scripts
LOG_LEVEL=0 spnd daily --json
```

## Short Aliases

Many options have short aliases for convenience:

| Long Option   | Short | Description         |
| ------------- | ----- | ------------------- |
| `--json`      | `-j`  | JSON output         |
| `--breakdown` | `-b`  | Per-model breakdown |
| `--offline`   | `-O`  | Offline mode        |
| `--timezone`  | `-z`  | Set timezone        |
| `--instances` | `-i`  | Group by project    |
| `--project`   | `-p`  | Filter project      |
| `--active`    | `-a`  | Active block only   |
| `--recent`    | `-r`  | Recent blocks       |

## Related Documentation

- [Environment Variables](/guide/environment-variables) - Configure via environment
- [Configuration Files](/guide/config-files) - Persistent configuration
- [Cost Calculation Modes](/guide/cost-modes) - Understanding cost modes
