use clap::{Args, Subcommand};
use serde_json::{Map, Value, json};

use crate::client::{OandaClient, OandaError, OandaResult, read_body};
use crate::commands::common::{
    ClientExtensionsArgs, MarketTimeInForce, PendingTimeInForce, PositionFill, ProtectionArgs,
    TriggerCondition, add_client_extensions, add_protection, validate_decimal, validate_instrument,
    validate_specifier,
};
use crate::config::Config;

#[derive(Subcommand)]
pub enum OrderCommand {
    /// Create an order from a raw JSON object
    Create {
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
    /// Create a typed market order
    Market(MarketOrderArgs),
    /// Create a typed limit order
    Limit(PendingOrderArgs),
    /// Create a typed stop order
    Stop(PendingOrderArgs),
    /// Get list of orders
    List {
        /// Comma-separated order IDs to filter
        #[arg(long)]
        ids: Option<String>,
        /// Order state filter (PENDING, FILLED, TRIGGERED, CANCELLED, ALL)
        #[arg(long)]
        state: Option<String>,
        /// Instrument filter
        #[arg(long)]
        instrument: Option<String>,
        /// Maximum number of orders to return
        #[arg(long)]
        count: Option<String>,
        /// Return orders before this ID
        #[arg(long)]
        before_id: Option<String>,
    },
    /// List all pending orders
    Pending,
    /// Get details for a single order
    Get {
        /// Order specifier (ID or client order ID with @prefix)
        order_specifier: String,
    },
    /// Replace an order with a raw JSON specification
    Replace {
        /// Order specifier to replace
        order_specifier: String,
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
    /// Cancel a pending order
    Cancel {
        /// Order specifier to cancel
        order_specifier: String,
    },
    /// Update client extensions on an order
    ClientExtensions {
        /// Order specifier
        order_specifier: String,
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
}

#[derive(Debug, Clone, Args)]
pub struct MarketOrderArgs {
    /// Instrument name, for example EUR_USD
    #[arg(long)]
    instrument: String,
    /// Signed decimal units: positive buys, negative sells
    #[arg(long, allow_hyphen_values = true)]
    units: String,
    /// Fill-or-kill or immediate-or-cancel
    #[arg(long, default_value = "FOK")]
    time_in_force: MarketTimeInForce,
    /// Position fill behavior
    #[arg(long, default_value = "DEFAULT")]
    position_fill: PositionFill,
    /// Worst acceptable fill price
    #[arg(long)]
    price_bound: Option<String>,
    #[command(flatten)]
    protection: ProtectionArgs,
    #[command(flatten)]
    client_extensions: ClientExtensionsArgs,
}

#[derive(Debug, Clone, Args)]
pub struct PendingOrderArgs {
    /// Instrument name, for example EUR_USD
    #[arg(long)]
    instrument: String,
    /// Signed decimal units: positive buys, negative sells
    #[arg(long, allow_hyphen_values = true)]
    units: String,
    /// Trigger price
    #[arg(long)]
    price: String,
    /// Good-till-cancelled, good-for-day, or good-till-date
    #[arg(long, default_value = "GTC")]
    time_in_force: PendingTimeInForce,
    /// RFC3339 expiry; required when --time-in-force GTD
    #[arg(long)]
    gtd_time: Option<String>,
    /// Position fill behavior
    #[arg(long, default_value = "DEFAULT")]
    position_fill: PositionFill,
    /// Price component used to trigger the order
    #[arg(long, default_value = "DEFAULT")]
    trigger_condition: TriggerCondition,
    #[command(flatten)]
    protection: ProtectionArgs,
    #[command(flatten)]
    client_extensions: ClientExtensionsArgs,
}

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    command: OrderCommand,
) -> OandaResult<()> {
    let account_id = config.require_account_id()?;
    match command {
        OrderCommand::Create { body } => {
            config.require_mutation_allowed()?;
            let body = read_body(body)?;
            client
                .post(&format!("/v3/accounts/{account_id}/orders"), body)
                .await
        }
        OrderCommand::Market(args) => {
            config.require_mutation_allowed()?;
            let body = market_order_body(args)?;
            client
                .post(&format!("/v3/accounts/{account_id}/orders"), body)
                .await
        }
        OrderCommand::Limit(args) => {
            config.require_mutation_allowed()?;
            let body = pending_order_body("LIMIT", args)?;
            client
                .post(&format!("/v3/accounts/{account_id}/orders"), body)
                .await
        }
        OrderCommand::Stop(args) => {
            config.require_mutation_allowed()?;
            let body = pending_order_body("STOP", args)?;
            client
                .post(&format!("/v3/accounts/{account_id}/orders"), body)
                .await
        }
        OrderCommand::List {
            ids,
            state,
            instrument,
            count,
            before_id,
        } => {
            if let Some(instrument) = &instrument {
                validate_instrument(instrument)?;
            }
            let mut query: Vec<(&str, &str)> = Vec::new();
            if let Some(value) = &ids {
                query.push(("ids", value));
            }
            if let Some(value) = &state {
                query.push(("state", value));
            }
            if let Some(value) = &instrument {
                query.push(("instrument", value));
            }
            if let Some(value) = &count {
                query.push(("count", value));
            }
            if let Some(value) = &before_id {
                query.push(("beforeID", value));
            }
            client
                .get(&format!("/v3/accounts/{account_id}/orders"), &query)
                .await
        }
        OrderCommand::Pending => {
            client
                .get(&format!("/v3/accounts/{account_id}/pendingOrders"), &[])
                .await
        }
        OrderCommand::Get { order_specifier } => {
            validate_specifier(&order_specifier, "order specifier")?;
            client
                .get(
                    &format!("/v3/accounts/{account_id}/orders/{order_specifier}"),
                    &[],
                )
                .await
        }
        OrderCommand::Replace {
            order_specifier,
            body,
        } => {
            config.require_mutation_allowed()?;
            validate_specifier(&order_specifier, "order specifier")?;
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{account_id}/orders/{order_specifier}"),
                    Some(body),
                )
                .await
        }
        OrderCommand::Cancel { order_specifier } => {
            config.require_mutation_allowed()?;
            validate_specifier(&order_specifier, "order specifier")?;
            client
                .put(
                    &format!("/v3/accounts/{account_id}/orders/{order_specifier}/cancel"),
                    None,
                )
                .await
        }
        OrderCommand::ClientExtensions {
            order_specifier,
            body,
        } => {
            config.require_mutation_allowed()?;
            validate_specifier(&order_specifier, "order specifier")?;
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{account_id}/orders/{order_specifier}/clientExtensions"),
                    Some(body),
                )
                .await
        }
    }
}

fn market_order_body(args: MarketOrderArgs) -> OandaResult<Value> {
    validate_instrument(&args.instrument)?;
    validate_decimal(&args.units, "units", true)?;
    let mut order = Map::from_iter([
        ("type".into(), json!("MARKET")),
        ("instrument".into(), json!(args.instrument)),
        ("units".into(), json!(args.units)),
        ("timeInForce".into(), json!(args.time_in_force.as_str())),
        ("positionFill".into(), json!(args.position_fill.as_str())),
    ]);
    if let Some(price_bound) = args.price_bound {
        validate_decimal(&price_bound, "price bound", false)?;
        order.insert("priceBound".into(), json!(price_bound));
    }
    add_protection(&mut order, args.protection)?;
    add_client_extensions(&mut order, args.client_extensions)?;
    Ok(json!({ "order": order }))
}

fn pending_order_body(order_type: &str, args: PendingOrderArgs) -> OandaResult<Value> {
    validate_instrument(&args.instrument)?;
    validate_decimal(&args.units, "units", true)?;
    validate_decimal(&args.price, "price", false)?;
    match (args.time_in_force, args.gtd_time.as_deref()) {
        (PendingTimeInForce::Gtd, None) => {
            return Err(OandaError::Validation(
                "--gtd-time is required when --time-in-force is GTD".into(),
            ));
        }
        (PendingTimeInForce::Gtd, Some(value)) if value.trim().is_empty() => {
            return Err(OandaError::Validation("--gtd-time cannot be empty".into()));
        }
        (_, Some(_)) if !matches!(args.time_in_force, PendingTimeInForce::Gtd) => {
            return Err(OandaError::Validation(
                "--gtd-time is only valid with --time-in-force GTD".into(),
            ));
        }
        _ => {}
    }

    let mut order = Map::from_iter([
        ("type".into(), json!(order_type)),
        ("instrument".into(), json!(args.instrument)),
        ("units".into(), json!(args.units)),
        ("price".into(), json!(args.price)),
        ("timeInForce".into(), json!(args.time_in_force.as_str())),
        ("positionFill".into(), json!(args.position_fill.as_str())),
        (
            "triggerCondition".into(),
            json!(args.trigger_condition.as_str()),
        ),
    ]);
    if let Some(gtd_time) = args.gtd_time {
        order.insert("gtdTime".into(), json!(gtd_time));
    }
    add_protection(&mut order, args.protection)?;
    add_client_extensions(&mut order, args.client_extensions)?;
    Ok(json!({ "order": order }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_order_is_typed_and_stringifies_numbers() {
        let body = market_order_body(MarketOrderArgs {
            instrument: "EUR_USD".into(),
            units: "-100".into(),
            time_in_force: MarketTimeInForce::Fok,
            position_fill: PositionFill::Default,
            price_bound: None,
            protection: ProtectionArgs {
                take_profit: Some("1.2".into()),
                stop_loss: None,
                trailing_stop_distance: None,
            },
            client_extensions: ClientExtensionsArgs {
                client_order_id: None,
                client_tag: None,
                client_comment: None,
            },
        })
        .unwrap();
        assert_eq!(body["order"]["type"], "MARKET");
        assert_eq!(body["order"]["units"], "-100");
        assert_eq!(body["order"]["takeProfitOnFill"]["price"], "1.2");
    }
}
