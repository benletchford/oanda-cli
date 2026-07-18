use clap::Subcommand;

use serde_json::json;

use crate::client::{OandaClient, OandaError, OandaResult, read_body};
use crate::commands::common::{validate_decimal, validate_specifier};
use crate::config::Config;

#[derive(Subcommand)]
pub enum TradeCommand {
    /// Get list of trades
    List {
        /// Comma-separated trade IDs to filter
        #[arg(long)]
        ids: Option<String>,
        /// Trade state filter (OPEN, CLOSED, CLOSE_WHEN_TRADEABLE, ALL)
        #[arg(long)]
        state: Option<String>,
        /// Instrument filter
        #[arg(long)]
        instrument: Option<String>,
        /// Maximum number of trades to return
        #[arg(long)]
        count: Option<String>,
        /// Return trades before this ID
        #[arg(long)]
        before_id: Option<String>,
    },
    /// Get list of open trades
    Open,
    /// Get details for a single trade
    Get {
        /// Trade specifier (ID or client trade ID with @prefix)
        trade_specifier: String,
    },
    /// Close a trade (fully or partially)
    Close {
        /// Trade specifier to close
        trade_specifier: String,
        /// Positive units to close; omit for a full close
        #[arg(long, conflicts_with = "body")]
        units: Option<String>,
        /// Raw JSON request body escape hatch
        #[arg(long, conflicts_with = "units")]
        body: Option<String>,
    },
    /// Update client extensions on a trade
    ClientExtensions {
        /// Trade specifier
        trade_specifier: String,
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
    /// Create, replace, or cancel dependent orders (TP/SL/TSL)
    Orders {
        /// Trade specifier
        trade_specifier: String,
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
}

pub async fn execute(client: &OandaClient, config: &Config, cmd: TradeCommand) -> OandaResult<()> {
    let id = config.require_account_id()?;
    match cmd {
        TradeCommand::List {
            ids,
            state,
            instrument,
            count,
            before_id,
        } => {
            let mut query: Vec<(&str, &str)> = vec![];
            if let Some(ref v) = ids {
                query.push(("ids", v));
            }
            if let Some(ref v) = state {
                query.push(("state", v));
            }
            if let Some(ref v) = instrument {
                query.push(("instrument", v));
            }
            if let Some(ref v) = count {
                query.push(("count", v));
            }
            if let Some(ref v) = before_id {
                query.push(("beforeID", v));
            }
            client
                .get(&format!("/v3/accounts/{id}/trades"), &query)
                .await
        }
        TradeCommand::Open => {
            client
                .get(&format!("/v3/accounts/{id}/openTrades"), &[])
                .await
        }
        TradeCommand::Get { trade_specifier } => {
            validate_specifier(&trade_specifier, "trade specifier")?;
            client
                .get(&format!("/v3/accounts/{id}/trades/{trade_specifier}"), &[])
                .await
        }
        TradeCommand::Close {
            trade_specifier,
            units,
            body,
        } => {
            validate_specifier(&trade_specifier, "trade specifier")?;
            let body = match (units, body) {
                (Some(units), None) => {
                    validate_decimal(&units, "units", false)?;
                    Some(json!({ "units": units }))
                }
                (None, Some(body)) => Some(read_body(Some(body))?),
                (None, None) => None,
                (Some(_), Some(_)) => {
                    return Err(OandaError::Validation(
                        "Use either --units or --body, not both".into(),
                    ));
                }
            };
            client
                .put(
                    &format!("/v3/accounts/{id}/trades/{trade_specifier}/close"),
                    body,
                )
                .await
        }
        TradeCommand::ClientExtensions {
            trade_specifier,
            body,
        } => {
            validate_specifier(&trade_specifier, "trade specifier")?;
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{id}/trades/{trade_specifier}/clientExtensions"),
                    Some(body),
                )
                .await
        }
        TradeCommand::Orders {
            trade_specifier,
            body,
        } => {
            validate_specifier(&trade_specifier, "trade specifier")?;
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{id}/trades/{trade_specifier}/orders"),
                    Some(body),
                )
                .await
        }
    }
}
