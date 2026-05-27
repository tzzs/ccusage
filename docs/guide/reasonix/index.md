# Reasonix Data Source

ccusage reads Reasonix token usage from `~/.reasonix/usage.jsonl` as one of its supported local data sources. Reasonix is a coding agent CLI that records per-turn token usage with pre-calculated costs in a flat JSONL file.

## What is Reasonix?

Reasonix is a coding (agent) CLI that tracks every model turn in `usage.jsonl`, recording prompt tokens, completion tokens, cache hit and miss counts, and per-turn cost in USD. ccusage reads this single file and aggregates it alongside other supported sources.

## Focused Views

```bash
# Recommended
bunx ccusage reasonix --help

# Alternative package runners
npx ccusage@latest reasonix --help
pnpm dlx ccusage reasonix --help
pnpx ccusage reasonix --help
```

## Data Source

The CLI scans for the Reasonix usage file:

| Source   | Default path              | Override        |
| -------- | ------------------------- | --------------- |
| Reasonix | `~/.reasonix/usage.jsonl` | `REASONIX_HOME` |

`REASONIX_HOME` can point to one directory or a comma-separated list of directories. Each directory is scanned for `usage.jsonl`.

## Report Views

```bash
# Show daily Reasonix usage
ccusage reasonix daily

# Show monthly Reasonix usage
ccusage reasonix monthly

# Show session-based Reasonix usage
ccusage reasonix session

# JSON output for automation
ccusage reasonix daily --json

# Custom Reasonix data directory
REASONIX_HOME=/path/to/.reasonix ccusage reasonix daily

# Filter by date range
ccusage reasonix daily --since 2026-05-01 --until 2026-05-28
```

## Cost Calculation

Reasonix records `costUsd` on every usage entry. ccusage uses these embedded costs directly and does not need the LiteLLM pricing database for Reasonix rows, so reports work offline without `--offline`.

## Token Mapping

Reasonix tracks tokens differently from Anthropic's API. ccusage maps them as follows for consistent reporting:

| Reasonix field     | ccusage field             |
| ------------------ | ------------------------- |
| `cacheMissTokens`  | `inputTokens`             |
| `completionTokens` | `outputTokens`            |
| `cacheHitTokens`   | `cacheReadTokens`         |
| —                  | `cacheCreationTokens` (0) |

Total tokens = cacheMissTokens + completionTokens + cacheHitTokens = promptTokens + completionTokens.

## Model Attribution

Reasonix records the model name (e.g. `deepseek-v4-pro`, `deepseek-v4-flash`) directly in each usage entry. Subagent turns (marked with `kind: "subagent"`) are included in the totals with the same model attribution.

## Environment Variables

| Variable        | Description                                                         |
| --------------- | ------------------------------------------------------------------- |
| `REASONIX_HOME` | Custom path, or comma-separated paths, to Reasonix data directories |
| `LOG_LEVEL`     | Adjust logging verbosity (0 silent … 5 trace)                       |

## Daily View

This view shows daily usage from Reasonix.

```bash
# Recommended (fastest)
bunx ccusage reasonix daily

# Using npx
npx ccusage@latest reasonix daily
```

### Options

| Flag         | Short | Description                                     |
| ------------ | ----- | ----------------------------------------------- |
| `--since`    |       | Start date filter (YYYY-MM-DD or YYYYMMDD)      |
| `--until`    |       | End date filter (YYYY-MM-DD or YYYYMMDD)        |
| `--timezone` | `-z`  | Override timezone for date grouping             |
| `--json`     | `-j`  | Emit structured JSON instead of a table         |
| `--compact`  |       | Force compact table layout for narrow terminals |

### Example Output

```text
┌────────────┬──────────────────────────┬──────────┬──────────┬──────────────┬────────────┬──────────────┬──────────────┐
│ Date       │ Models                   │    Input │   Output │ Cache Create │ Cache Read │ Total Tokens │   Cost (USD) │
├────────────┼──────────────────────────┼──────────┼──────────┼──────────────┼────────────┼──────────────┼──────────────┤
│ 2026-05-28 │ - deepseek-v4-flash      │  223,054 │   54,155 │            0 │  8,980,736 │    9,257,945 │        $0.17 │
│            │ - deepseek-v4-pro        │          │          │              │            │              │              │
├────────────┼──────────────────────────┼──────────┼──────────┼──────────────┼────────────┼──────────────┼──────────────┤
│ Total      │                          │  223,054 │   54,155 │            0 │  8,980,736 │    9,257,945 │        $0.17 │
└────────────┴──────────────────────────┴──────────┴──────────┴──────────────┴────────────┴──────────────┴──────────────┘
```

### JSON Output

Use `--json` for automation and scripting:

```bash
ccusage reasonix daily --json
```

Returns structured data:

<!-- eslint-skip -->

```json
{
	"daily": [
		{
			"date": "2026-05-28",
			"inputTokens": 223054,
			"outputTokens": 54155,
			"cacheCreationTokens": 0,
			"cacheReadTokens": 8980736,
			"totalTokens": 9257945,
			"totalCost": 0.17,
			"modelsUsed": ["deepseek-v4-pro", "deepseek-v4-flash"]
		}
	],
	"totals": {
		"inputTokens": 223054,
		"outputTokens": 54155,
		"cacheCreationTokens": 0,
		"cacheReadTokens": 8980736,
		"totalTokens": 9257945,
		"totalCost": 0.17
	}
}
```

## Monthly View

This view shows monthly usage from Reasonix.

```bash
ccusage reasonix monthly
ccusage reasonix monthly --json
ccusage reasonix monthly --since 2026-01-01 --until 2026-12-31
```

## Session View

This view shows usage grouped by individual Reasonix sessions. Session IDs come from the `session` field in each usage record.

```bash
ccusage reasonix session
ccusage reasonix session --json
ccusage reasonix session --since 2026-05-28
```

### Example Output

```text
┌────────────────────┬──────────────────────┬──────────┬──────────┬──────────────┬────────────┬──────────────┬──────────────┐
│ Session            │ Models               │    Input │   Output │ Cache Create │ Cache Read │ Total Tokens │   Cost (USD) │
├────────────────────┼──────────────────────┼──────────┼──────────┼──────────────┼────────────┼──────────────┼──────────────┤
│ code-blissful-…    │ - deepseek-v4-flash  │  223,054 │   54,155 │            0 │  8,980,736 │    9,257,945 │        $0.17 │
│                    │ - deepseek-v4-pro    │          │          │              │            │              │              │
├────────────────────┼──────────────────────┼──────────┼──────────┼──────────────┼────────────┼──────────────┼──────────────┤
│ Total              │                      │  223,054 │   54,155 │            0 │  8,980,736 │    9,257,945 │        $0.17 │
└────────────────────┴──────────────────────┴──────────┴──────────┴──────────────┴────────────┴──────────────┴──────────────┘
```

## Related

- [ccusage](https://github.com/ryoppippi/ccusage) - Main usage analysis tool for coding (agent) CLIs
