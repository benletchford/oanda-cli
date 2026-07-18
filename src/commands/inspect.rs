use serde_json::{Value, json};

use crate::client::{OandaClient, OandaResult};
use crate::commands::common::validate_instrument;
use crate::config::Config;

pub async fn execute(client: &OandaClient, config: &Config, instrument: String) -> OandaResult<()> {
    validate_instrument(&instrument)?;
    let account_id = config.require_account_id()?;
    let instruments_path = format!("/v3/accounts/{account_id}/instruments");
    let pricing_path = format!("/v3/accounts/{account_id}/pricing");
    let position_path = format!("/v3/accounts/{account_id}/positions/{instrument}");
    let pending_path = format!("/v3/accounts/{account_id}/pendingOrders");
    let trades_path = format!("/v3/accounts/{account_id}/trades");
    let instrument_query = [("instruments", instrument.as_str())];
    let trades_query = [("state", "OPEN"), ("instrument", instrument.as_str())];

    let (
        instrument_response,
        pricing_response,
        position_response,
        pending_response,
        trades_response,
    ) = tokio::try_join!(
        client.get_json(&instruments_path, &instrument_query),
        client.get_json(&pricing_path, &instrument_query),
        client.get_json(&position_path, &[]),
        client.get_json(&pending_path, &[]),
        client.get_json(&trades_path, &trades_query),
    )?;

    let output = json!({
        "instrument": &instrument,
        "environment": config.environment,
        "instrumentDetails": first_from(&instrument_response, "instruments"),
        "price": first_from(&pricing_response, "prices"),
        "position": position_response.get("position").cloned().unwrap_or(position_response),
        "pendingOrders": matching_items(&pending_response, "orders", &instrument),
        "openTrades": matching_items(&trades_response, "trades", &instrument),
    });
    client.print_json(&output)
}

fn first_from(value: &Value, key: &str) -> Value {
    value
        .get(key)
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(Value::Null)
}

fn matching_items(value: &Value, key: &str, instrument: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| item.get("instrument").and_then(Value::as_str) == Some(instrument))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_context_by_instrument() {
        let value = json!({"orders": [
            {"id": "1", "instrument": "EUR_USD"},
            {"id": "2", "instrument": "USD_JPY"}
        ]});
        assert_eq!(matching_items(&value, "orders", "EUR_USD").len(), 1);
    }
}
