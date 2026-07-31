# Goose Data Source (Experimental)

Goose support is experimental. spnd can read Goose session usage from its local SQLite database. Goose is a local AI agent, and spnd only opens the database read-only.

## Quick Start

```bash
spnd goose daily
spnd goose monthly
spnd goose session
```

Goose is also included in unified reports when a supported database is detected:

```bash
spnd daily
```

## Data Locations

By default, spnd checks these Goose database locations:

```text
~/.local/share/goose/sessions/sessions.db
~/Library/Application Support/goose/sessions/sessions.db
~/.local/share/Block/goose/sessions/sessions.db
```

Set `GOOSE_PATH_ROOT` when Goose is stored somewhere else:

```bash
GOOSE_PATH_ROOT="/path/to/goose" spnd goose daily
```

With `GOOSE_PATH_ROOT` set, spnd reads:

```text
$GOOSE_PATH_ROOT/data/sessions/sessions.db
```

## Token Mapping

spnd reads Goose session rows with model configuration and token columns:

| Goose column                                        | spnd field          |
| --------------------------------------------------- | ------------------- |
| `accumulated_input_tokens` or `input_tokens`        | Input tokens        |
| `accumulated_output_tokens` or `output_tokens`      | Output tokens       |
| `accumulated_total_tokens` or `total_tokens`        | Total tokens        |
| `total - input - output`, when positive             | Reasoning/extra use |
| `model_config_json.model_name`                      | Model               |
| `provider_name`, when present, otherwise model hint | Provider            |

Goose does not expose cache read/write token columns in this database, so cache columns are reported as zero.

## Cost Calculation

Goose rows do not store recorded USD cost, so spnd estimates cost from token counts and LiteLLM pricing. Any positive total-token remainder beyond input and output is treated as output-priced usage for cost estimation.

Use `--offline` to rely on cached pricing data:

```bash
spnd goose daily --offline
```

## Troubleshooting

If no Goose data appears, check that the SQLite database exists at one of the default paths or set `GOOSE_PATH_ROOT`.

```bash
GOOSE_PATH_ROOT="/path/to/goose" spnd goose session --json
```
