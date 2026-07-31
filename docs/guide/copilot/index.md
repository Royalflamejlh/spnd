# GitHub Copilot CLI Data Source (Beta)

> GitHub Copilot CLI support is experimental. The adapter reads local OpenTelemetry JSONL files only.

spnd can read GitHub Copilot CLI OpenTelemetry file exports as one of its supported local data sources. It uses the same reporting experience as the rest of spnd: responsive tables, JSON output, LiteLLM-based pricing, cache token accounting, and all-source aggregation.

## Focused Views

::: code-group

```bash [bunx (Recommended)]
bunx @spnd/spnd copilot --help
```

```bash [npx]
npx @spnd/spnd@latest copilot --help
```

```bash [pnpm]
pnpm dlx @spnd/spnd copilot --help
```

:::

## Data Source

The CLI reads Copilot OpenTelemetry JSONL files from `~/.copilot/otel/*.jsonl` and also includes the explicit file pointed to by `COPILOT_OTEL_FILE_EXPORTER_PATH`.

Enable these variables before starting or resuming a Copilot CLI session. Sessions that ran without OpenTelemetry file export enabled do not produce local JSONL usage data for spnd to read.

```bash
export COPILOT_OTEL_ENABLED=true
export COPILOT_OTEL_EXPORTER_TYPE=file
mkdir -p "$HOME/.copilot/otel"
export COPILOT_OTEL_FILE_EXPORTER_PATH="$HOME/.copilot/otel/copilot-otel-$(date +%Y%m%d-%H%M%S).jsonl"
```

```text
~/.copilot/
└── otel/
    └── *.jsonl
```

## Report Views

| Focused view           | Description                        | See also                                |
| ---------------------- | ---------------------------------- | --------------------------------------- |
| `spnd copilot daily`   | Aggregate usage by date            | [Daily Usage](/guide/daily-reports)     |
| `spnd copilot monthly` | Aggregate usage by month           | [Monthly Usage](/guide/monthly-reports) |
| `spnd copilot session` | Group usage by Copilot session IDs | [Session Usage](/guide/session-reports) |

These views support `--json` for structured output, `--compact` for narrow terminals, and `--offline` for cached pricing data.

## What Gets Calculated

- **Token usage** - chat spans are preferred, with inference logs and agent-turn logs used as fallbacks.
- **Cache tokens** - cache read and cache creation token attributes are counted when present.
- **Reasoning tokens** - reasoning output tokens are included in total tokens and cost calculation.
- **Pricing** - costs are calculated from LiteLLM pricing data using the reported model name.

## Environment Variables

| Variable                          | Description                                          |
| --------------------------------- | ---------------------------------------------------- |
| `COPILOT_OTEL_FILE_EXPORTER_PATH` | Explicit Copilot OpenTelemetry JSONL file to include |
| `LOG_LEVEL`                       | Adjust verbosity (0 silent ... 5 trace)              |

## Troubleshooting

::: details No Copilot usage data found
Ensure OpenTelemetry file export is enabled and the exporter path points to an existing `.jsonl` file, or place exported `.jsonl` files under `~/.copilot/otel/`.

If you are using `copilot --resume`, set the OpenTelemetry environment variables before running the resume command. Earlier activity from sessions started without file export cannot be recovered by spnd.
:::

::: details Costs showing as $0.00
If a model is not in LiteLLM's database, the cost will be $0.00. [Open an issue](https://github.com/ccusage/ccusage/issues/new) to request alias support.
:::
