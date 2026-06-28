use clap::Subcommand;

use crate::client::{OandaClient, read_body};
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
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
}

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    cmd: PositionCommand,
) -> Result<(), String> {
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
            client
                .get(&format!("/v3/accounts/{id}/positions/{instrument}"), &[])
                .await
        }
        PositionCommand::Close { instrument, body } => {
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{id}/positions/{instrument}/close"),
                    Some(body),
                )
                .await
        }
    }
}
