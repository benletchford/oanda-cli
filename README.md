# oanda-cli

A small, machine-friendly CLI and Rust library for the [OANDA v20 REST API](https://developer.oanda.com/rest-live-v20/introduction/). Successful responses are JSON, streaming responses are NDJSON, and runtime errors are structured JSON on stderr.

The CLI keeps raw endpoint access available while adding typed commands, validation, dry runs, explicit live-trading safeguards, bounded I/O, and stable exit codes.

## Install

From crates.io:

```sh
cargo install --locked oanda-cli
```

Or from this checkout:

```sh
cargo install --locked --path .
```

GitHub releases include versioned archives and SHA-256 checksum files for Linux x86-64/ARM64 (static musl builds), macOS x86-64/ARM64, and Windows x86-64.

## Configuration

Flags take precedence over environment variables:

| Flag | Environment variable | Description |
|---|---|---|
| `--token` | `OANDA_ACCESS_TOKEN` | OANDA personal access token |
| `--account-id` | `OANDA_ACCOUNT_ID` | Account ID required by account-scoped endpoints |
| `--environment` | `OANDA_ENVIRONMENT` | `practice` or `live` |
| `--datetime-format` | `OANDA_DATETIME_FORMAT` | `RFC3339` or `UNIX` |
| `--request-timeout-secs` | `OANDA_REQUEST_TIMEOUT_SECS` | Non-streaming timeout; default 30 seconds |
| `--connect-timeout-secs` | `OANDA_CONNECT_TIMEOUT_SECS` | Connection timeout; default 10 seconds |

Use `OANDA_ACCESS_TOKEN`; `OANDA_TOKEN` is not read. Prefer the environment variable to `--token`, which can expose a token through shell history and process listings.

```sh
export OANDA_ACCESS_TOKEN="your-api-token"
export OANDA_ACCOUNT_ID="101-001-12345678-001"
export OANDA_ENVIRONMENT="practice"
```

Generate a token in OANDA fxTrade under **My Services > Manage API Access**.

### Environment and mutation safeguards

Read-only commands default to the practice environment when neither the flag nor environment variable is set. Every mutation requires an explicit environment through `--environment` or `OANDA_ENVIRONMENT`.

Live mutations additionally require `--confirm-live`:

```sh
oanda --environment live --confirm-live order cancel 6357
```

This is an acknowledgement guard, not an interactive prompt. Scripts remain deterministic.

## Agent-friendly interfaces

### Typed mutations

Common mutations avoid shell-quoted JSON and keep decimal quantities/prices as JSON strings in OANDA requests:

```sh
# Positive units buy; negative units sell
oanda --environment practice order market \
  --instrument EUR_USD \
  --units 100 \
  --position-fill DEFAULT

oanda --environment practice order limit \
  --instrument EUR_USD \
  --units -100 \
  --price 1.1000 \
  --take-profit 1.0500 \
  --stop-loss 1.1500

oanda --environment practice trade close 6357 --units 50
oanda --environment practice position close EUR_USD --long-units ALL
```

Typed order commands support `market`, `limit`, and `stop`, along with position-fill, time-in-force, trigger-condition, price-bound, take-profit, stop-loss, trailing-stop, and client-extension options. Run a command with `--help` for its exact parameters.

### Dry runs

`--dry-run` validates a mutation and prints its method, endpoint, environment, query, and redacted body without making a network request. A token is not required, but the account ID and explicit environment still are.

```sh
oanda --environment practice --account-id 101-001-12345678-001 --dry-run \
  order market --instrument EUR_USD --units 100
```

```json
{
  "body": {
    "order": {
      "instrument": "EUR_USD",
      "positionFill": "DEFAULT",
      "timeInForce": "FOK",
      "type": "MARKET",
      "units": "100"
    }
  },
  "dryRun": true,
  "endpoint": "/v3/accounts/101-001-12345678-001/orders",
  "environment": "practice",
  "method": "POST",
  "mutation": true,
  "query": []
}
```

### Machine-readable schema

`oanda schema --json` needs no credentials and describes commands, parameters, mutation classification, examples, configuration, output formats, and exit codes.

### Consolidated inspection

`inspect` concurrently gathers instrument metadata, current price, position, pending orders, and open trades for one instrument:

```sh
oanda inspect EUR_USD
```

### Structured errors and exit codes

Runtime errors are emitted as one JSON object on stderr:

```json
{
  "error": {
    "exitCode": 3,
    "kind": "configuration",
    "message": "Live mutations require --confirm-live"
  }
}
```

| Code | Meaning |
|---:|---|
| 0 | Success |
| 2 | CLI usage or validation error |
| 3 | Configuration or authentication error |
| 4 | Network failure or timeout |
| 5 | OANDA API rejection |
| 6 | Invalid response or local I/O failure |

CLI usage errors use the same JSON envelope with kind `validation` and exit code 2. `--help` and `--version` retain their normal text output.

## Raw endpoint commands

Raw JSON remains the escape hatch for OANDA fields that typed commands do not expose. Mutation bodies must be JSON objects and are limited to 1 MiB. If `--body` is omitted where documented, the CLI reads JSON from stdin.

```sh
oanda --environment practice order create --body \
  '{"order":{"type":"MARKET","instrument":"EUR_USD","units":"100","timeInForce":"FOK","positionFill":"DEFAULT"}}'

oanda --environment practice order create < order.json
```

### Account

```sh
oanda account list
oanda account get
oanda account summary
oanda account instruments --instruments EUR_USD,USD_JPY
oanda account changes --since-transaction-id 6356
oanda --environment practice account configure --body '{"alias":"Primary"}'
```

### Instrument

```sh
oanda instrument candles EUR_USD --granularity H1 --count 10
oanda instrument candles EUR_USD --granularity D \
  --from 2026-01-01T00:00:00Z --to 2026-02-01T00:00:00Z
```

### Order

```sh
oanda order list --state PENDING
oanda order pending
oanda order get 6357
oanda --environment practice order replace 6357 --body '{"order":{"type":"LIMIT","instrument":"EUR_USD","units":"100","price":"1.1000","timeInForce":"GTC","positionFill":"DEFAULT"}}'
oanda --environment practice order cancel 6357
```

Use a numeric ID or `@client_id` as an order specifier.

### Trade

```sh
oanda trade list
oanda trade open
oanda trade get 6357
oanda --environment practice trade close 6357
oanda --environment practice trade orders 6357 --body '{"takeProfit":{"price":"1.1500"},"stopLoss":{"price":"1.0500"}}'
```

### Position

```sh
oanda position list
oanda position open
oanda position get EUR_USD
oanda --environment practice position close EUR_USD --short-units ALL
```

### Pricing

```sh
oanda pricing get --instruments EUR_USD,USD_JPY
oanda pricing candles EUR_USD --granularity M5 --count 20
oanda pricing candles-latest --candle-specifications EUR_USD:S5:MBA
oanda pricing stream --instruments EUR_USD,USD_JPY
```

Streams do not use the overall request timeout. Press Ctrl-C for clean cancellation.

### Transaction

```sh
oanda transaction list --from 2026-01-01T00:00:00Z --to 2026-02-01T00:00:00Z
oanda transaction get 6357
oanda transaction idrange --from 6356 --to 6358
oanda transaction sinceid --id 6356 --type ORDER,TRADE
oanda transaction stream
```

### Labs (unofficial)

The public OANDA Labs GraphQL command does not require a token or account ID:

```sh
oanda labs book EURUSD --book-type order
oanda labs book USDJPY --book-type position --recent-hours 6
```

## Piping and scripting

```sh
oanda pricing get --instruments EUR_USD | jq -r '.prices[0].bids[0].price'
oanda trade open | jq -r '.trades[].id'
oanda instrument candles EUR_USD --granularity D --count 365 > candles.json
```

Add `--pretty` to format non-streaming JSON for inspection.

## Rust library

The package exposes a library crate named `oanda_cli`:

```toml
[dependencies]
oanda-cli = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use oanda_cli::{CandlesParams, Config, OandaClient, PricingGetParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("OANDA_ACCESS_TOKEN")?;
    let config = Config::new(token, std::env::var("OANDA_ACCOUNT_ID")?);
    let client = OandaClient::new(&config)?;

    let summary = client.account().summary().await?;
    let prices = client
        .pricing()
        .get_with(
            "EUR_USD,USD_JPY",
            PricingGetParams::default().include_home_conversions(),
        )
        .await?;
    let candles = client
        .instrument("EUR_USD")
        .candles_with(CandlesParams::default().granularity("H1").count(10))
        .await?;

    println!("{summary}\n{prices}\n{candles}");
    Ok(())
}
```

Resource helpers cover accounts, instruments, orders, trades, positions, pricing, and transactions. `get_json`, `post_json`, `put_json`, `patch_json`, `request_json`, and `stream_response` remain lower-level escape hatches.

## Development and releases

CI runs formatting, strict Clippy, tests on Linux/macOS/Windows, package validation, a dependency audit, and a guard against native-TLS dependencies. Pull-request titles use Conventional Commits; see [CONTRIBUTING.md](CONTRIBUTING.md).

Release Please maintains release PRs, `CHANGELOG.md`, crate versions, tags, and GitHub releases. Merging a release PR publishes to crates.io using `CARGO_REGISTRY_TOKEN` and uploads versioned binaries with SHA-256 checksums.

## License

[MIT](LICENSE)
