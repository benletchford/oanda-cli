use clap::Subcommand;

use serde_json::json;

use crate::client::{OandaClient, OandaError, OandaResult, read_body};
use crate::commands::common::{validate_decimal, validate_instrument};
use crate::config::Config;

#[derive(Subcommand)]
pub enum PositionCommand {
    /// List all positions for an account
    List,
    /// List all open positions
    Open,
    /// Get position for a single instrument
    Get {
        /// Instrument name (e.g. EUR_USD)
        instrument: String,
    },
    /// Close out an open position
    Close {
        /// Instrument name (e.g. EUR_USD)
        instrument: String,
        /// Long units to close: ALL, NONE, or a positive decimal
        #[arg(long, conflicts_with = "body")]
        long_units: Option<String>,
        /// Short units to close: ALL, NONE, or a positive decimal
        #[arg(long, conflicts_with = "body")]
        short_units: Option<String>,
        /// Raw JSON request body escape hatch (reads from stdin when no typed units are given)
        #[arg(long, conflicts_with_all = ["long_units", "short_units"])]
        body: Option<String>,
    },
}

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    cmd: PositionCommand,
) -> OandaResult<()> {
    let id = config.require_account_id()?;
    match cmd {
        PositionCommand::List => {
            client
                .get(&format!("/v3/accounts/{id}/positions"), &[])
                .await
        }
        PositionCommand::Open => {
            client
                .get(&format!("/v3/accounts/{id}/openPositions"), &[])
                .await
        }
        PositionCommand::Get { instrument } => {
            validate_instrument(&instrument)?;
            client
                .get(&format!("/v3/accounts/{id}/positions/{instrument}"), &[])
                .await
        }
        PositionCommand::Close {
            instrument,
            long_units,
            short_units,
            body,
        } => {
            validate_instrument(&instrument)?;
            let body = if long_units.is_some() || short_units.is_some() {
                let long_units = long_units.unwrap_or_else(|| "NONE".into());
                let short_units = short_units.unwrap_or_else(|| "NONE".into());
                validate_close_units(&long_units)?;
                validate_close_units(&short_units)?;
                if long_units == "NONE" && short_units == "NONE" {
                    return Err(OandaError::Validation(
                        "At least one of --long-units or --short-units must close units".into(),
                    ));
                }
                json!({ "longUnits": long_units, "shortUnits": short_units })
            } else {
                read_body(body)?
            };
            client
                .put(
                    &format!("/v3/accounts/{id}/positions/{instrument}/close"),
                    Some(body),
                )
                .await
        }
    }
}

fn validate_close_units(value: &str) -> OandaResult<()> {
    if matches!(value, "ALL" | "NONE") {
        Ok(())
    } else {
        validate_decimal(value, "position close units", false)
    }
}
