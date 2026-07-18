use serde_json::{Value, json};

pub fn document() -> Value {
    let commands = vec![
        command("account list", "read", &[], "oanda account list"),
        command("account get", "read", &[], "oanda account get"),
        command("account summary", "read", &[], "oanda account summary"),
        command(
            "account instruments",
            "read",
            &["--instruments"],
            "oanda account instruments --instruments EUR_USD",
        ),
        command(
            "account configure",
            "mutation",
            &["--body|stdin"],
            "oanda --environment practice account configure --body '{\"alias\":\"Primary\"}'",
        ),
        command(
            "account changes",
            "read",
            &["--since-transaction-id"],
            "oanda account changes --since-transaction-id 6356",
        ),
        command(
            "instrument candles",
            "read",
            &["instrument", "--granularity", "--count", "--from", "--to"],
            "oanda instrument candles EUR_USD --granularity H1 --count 10",
        ),
        command(
            "order create",
            "mutation",
            &["--body|stdin"],
            "oanda --environment practice order create --body '{\"order\":{...}}'",
        ),
        command(
            "order market",
            "mutation",
            &[
                "--instrument",
                "--units",
                "--time-in-force",
                "--position-fill",
                "--price-bound",
                "--take-profit",
                "--stop-loss",
                "--trailing-stop-distance",
            ],
            "oanda --environment practice order market --instrument EUR_USD --units 100 --position-fill DEFAULT",
        ),
        command(
            "order limit",
            "mutation",
            &[
                "--instrument",
                "--units",
                "--price",
                "--time-in-force",
                "--gtd-time",
                "--position-fill",
                "--trigger-condition",
            ],
            "oanda --environment practice order limit --instrument EUR_USD --units 100 --price 1.1000",
        ),
        command(
            "order stop",
            "mutation",
            &[
                "--instrument",
                "--units",
                "--price",
                "--time-in-force",
                "--gtd-time",
                "--position-fill",
                "--trigger-condition",
            ],
            "oanda --environment practice order stop --instrument EUR_USD --units -100 --price 1.0500",
        ),
        command(
            "order list",
            "read",
            &["--ids", "--state", "--instrument", "--count", "--before-id"],
            "oanda order list --state PENDING",
        ),
        command("order pending", "read", &[], "oanda order pending"),
        command(
            "order get",
            "read",
            &["order-specifier"],
            "oanda order get 6357",
        ),
        command(
            "order replace",
            "mutation",
            &["order-specifier", "--body|stdin"],
            "oanda --environment practice order replace 6357 --body '{\"order\":{...}}'",
        ),
        command(
            "order cancel",
            "mutation",
            &["order-specifier"],
            "oanda --environment practice order cancel 6357",
        ),
        command(
            "order client-extensions",
            "mutation",
            &["order-specifier", "--body|stdin"],
            "oanda --environment practice order client-extensions 6357 --body '{\"clientExtensions\":{...}}'",
        ),
        command(
            "trade list",
            "read",
            &["--ids", "--state", "--instrument", "--count", "--before-id"],
            "oanda trade list --state OPEN",
        ),
        command("trade open", "read", &[], "oanda trade open"),
        command(
            "trade get",
            "read",
            &["trade-specifier"],
            "oanda trade get 6357",
        ),
        command(
            "trade close",
            "mutation",
            &["trade-specifier", "--units|--body"],
            "oanda --environment practice trade close 6357 --units 50",
        ),
        command(
            "trade client-extensions",
            "mutation",
            &["trade-specifier", "--body|stdin"],
            "oanda --environment practice trade client-extensions 6357 --body '{\"clientExtensions\":{...}}'",
        ),
        command(
            "trade orders",
            "mutation",
            &["trade-specifier", "--body|stdin"],
            "oanda --environment practice trade orders 6357 --body '{\"takeProfit\":{...}}'",
        ),
        command("position list", "read", &[], "oanda position list"),
        command("position open", "read", &[], "oanda position open"),
        command(
            "position get",
            "read",
            &["instrument"],
            "oanda position get EUR_USD",
        ),
        command(
            "position close",
            "mutation",
            &[
                "instrument",
                "--long-units",
                "--short-units",
                "--body|stdin",
            ],
            "oanda --environment practice position close EUR_USD --long-units ALL",
        ),
        command(
            "pricing get",
            "read",
            &["--instruments", "--since", "--include-home-conversions"],
            "oanda pricing get --instruments EUR_USD,USD_JPY",
        ),
        command(
            "pricing stream",
            "stream",
            &["--instruments", "--snapshot", "--include-home-conversions"],
            "oanda pricing stream --instruments EUR_USD",
        ),
        command(
            "pricing candles",
            "read",
            &["instrument", "--granularity", "--count", "--from", "--to"],
            "oanda pricing candles EUR_USD --granularity M5 --count 20",
        ),
        command(
            "pricing candles-latest",
            "read",
            &["--candle-specifications"],
            "oanda pricing candles-latest --candle-specifications EUR_USD:S5:MBA",
        ),
        command(
            "transaction list",
            "read",
            &["--from", "--to", "--page-size", "--type"],
            "oanda transaction list --from 2026-01-01T00:00:00Z",
        ),
        command(
            "transaction get",
            "read",
            &["transaction-id"],
            "oanda transaction get 6357",
        ),
        command(
            "transaction idrange",
            "read",
            &["--from", "--to", "--type"],
            "oanda transaction idrange --from 6356 --to 6358",
        ),
        command(
            "transaction sinceid",
            "read",
            &["--id", "--type"],
            "oanda transaction sinceid --id 6356",
        ),
        command(
            "transaction stream",
            "stream",
            &[],
            "oanda transaction stream",
        ),
        command("inspect", "read", &["instrument"], "oanda inspect EUR_USD"),
        command(
            "labs book",
            "read",
            &["instrument", "--book-type", "--recent-hours"],
            "oanda labs book EURUSD --book-type order",
        ),
        command("schema", "local", &["--json"], "oanda schema --json"),
    ];

    json!({
        "schemaVersion": 2,
        "name": "oanda",
        "version": env!("CARGO_PKG_VERSION"),
        "output": { "success": "JSON", "streams": "NDJSON", "errors": "JSON on stderr" },
        "configuration": {
            "accessToken": { "flag": "--token", "environment": "OANDA_ACCESS_TOKEN" },
            "accountId": { "flag": "--account-id", "environment": "OANDA_ACCOUNT_ID" },
            "environment": { "flag": "--environment", "variable": "OANDA_ENVIRONMENT", "values": ["practice", "live"], "default": "practice" },
            "dryRun": "--dry-run"
        },
        "exitCodes": {
            "0": "success",
            "2": "usage_or_validation",
            "3": "configuration_or_authentication",
            "4": "network_or_timeout",
            "5": "api_rejection",
            "6": "response_or_io"
        },
        "commands": commands,
    })
}

fn command(path: &str, operation: &str, parameters: &[&str], example: &str) -> Value {
    json!({
        "path": path,
        "operation": operation,
        "mutation": operation == "mutation",
        "parameters": parameters,
        "example": example,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mutation_is_classified() {
        let document = document();
        let commands = document["commands"].as_array().unwrap();
        assert!(
            commands
                .iter()
                .any(|command| command["path"] == "order market" && command["mutation"] == true)
        );
        assert!(
            commands
                .iter()
                .all(|command| command["path"].is_string() && command["example"].is_string())
        );
    }
}
