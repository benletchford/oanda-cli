# oanda-cli

A thin CLI wrapper around the [OANDA v20 REST API](https://developer.oanda.com/rest-live-v20/introduction/). Every endpoint maps 1:1 to a subcommand. Outputs raw JSON.

## Install

```sh
cargo install oanda-cli
```

Or build from source:

```sh
cargo install --path .
```

## Configuration

Authentication is via CLI flags or environment variables (first wins):

| Flag | Env var | Description |
|---|---|---|
| `--token` | `OANDA_TOKEN` | API bearer token (required for v20 endpoints) |
| `--account-id` | `OANDA_ACCOUNT_ID` | Account ID (required for most endpoints) |
| `--environment` | `OANDA_ENVIRONMENT` | `practice` (default) or `live` |
| `--datetime-format` | `OANDA_DATETIME_FORMAT` | `RFC3339` or `UNIX` |

Generate an API token at [OANDA fxTrade](https://www.oanda.com/account/tpa/personal_token) under My Services > Manage API Access.

```sh
export OANDA_TOKEN="your-api-token"
export OANDA_ACCOUNT_ID="101-001-12345678-001"
```

## Usage

```
oanda [OPTIONS] <COMMAND>
```

Add `--pretty` to any command for formatted JSON output.

### Account

```sh
oanda account list
oanda account get
oanda account summary
oanda account instruments
oanda account instruments --instruments EUR_USD,USD_JPY
oanda account configure --body '{"alias":"Primary"}'
oanda account changes --since-transaction-id 6356
```

### Instrument

```sh
oanda instrument candles EUR_USD
oanda instrument candles EUR_USD --granularity H1 --count 10
oanda instrument candles EUR_USD --granularity D --from 2024-01-01T00:00:00Z --to 2024-02-01T00:00:00Z
```

### Order

```sh
oanda order list
oanda order pending
oanda order get 6357

# Create a market order
oanda order create --body '{"order":{"type":"MARKET","instrument":"EUR_USD","units":"100","timeInForce":"FOK","positionFill":"DEFAULT"}}'

# Create a limit order from a file
cat order.json | oanda order create

oanda order replace 6357 --body '{"order":{"type":"LIMIT","instrument":"EUR_USD","units":"100","price":"1.1000","timeInForce":"GTC"}}'
oanda order cancel 6357
oanda order client-extensions 6357 --body '{"clientExtensions":{"comment":"my order"}}'
```

### Trade

```sh
oanda trade list
oanda trade open
oanda trade get 6357
oanda trade close 6357
oanda trade close 6357 --body '{"units":"50"}'
oanda trade client-extensions 6357 --body '{"clientExtensions":{"comment":"my trade"}}'
oanda trade orders 6357 --body '{"takeProfit":{"price":"1.1500"},"stopLoss":{"price":"1.0500"}}'
```

### Position

```sh
oanda position list
oanda position open
oanda position get EUR_USD
oanda position close EUR_USD --body '{"longUnits":"ALL"}'
```

### Pricing

```sh
oanda pricing get --instruments EUR_USD,USD_JPY
oanda pricing get --instruments EUR_USD --include-units-available
oanda pricing candles EUR_USD --granularity M5 --count 20
oanda pricing candles-latest --candle-specifications EUR_USD:S5:MBA

# Stream live prices (newline-delimited JSON)
oanda pricing stream --instruments EUR_USD,USD_JPY
```

### Transaction

```sh
oanda transaction list --from 2024-01-01T00:00:00Z --to 2024-02-01T00:00:00Z
oanda transaction get 6357
oanda transaction idrange --from 6356 --to 6358
oanda transaction sinceid --id 6356
oanda transaction sinceid --id 6356 --type ORDER,TRADE

# Stream transactions in real-time
oanda transaction stream
```

### Labs (unofficial)

Fetches order/position book data from OANDA's public Labs API. No authentication required.

```sh
oanda labs book EURUSD --book-type order
oanda labs book USDJPY --book-type position
oanda labs book EURUSD --book-type order --recent-hours 6
```

Supported instruments: `AUDJPY`, `AUDUSD`, `EURAUD`, `EURCHF`, `EURGBP`, `EURJPY`, `EURUSD`, `GBPCHF`, `GBPJPY`, `GBPUSD`, `NZDUSD`, `USDCAD`, `USDCHF`, `USDJPY`.

## Piping and scripting

Output is plain JSON, designed for piping into tools like `jq`:

```sh
# Get the current bid price for EUR/USD
oanda pricing get --instruments EUR_USD | jq '.prices[0].bids[0].price'

# List all open trade IDs
oanda trade open | jq '.trades[].id'

# Save candle data to a file
oanda instrument candles EUR_USD --granularity D --count 365 > candles.json
```

## License

MIT
