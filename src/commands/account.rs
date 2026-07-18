use clap::Subcommand;

use crate::client::{OandaClient, OandaResult, read_body};
use crate::config::Config;

#[derive(Subcommand)]
pub enum AccountCommand {
    /// Get list of all authorized accounts
    List,
    /// Get comprehensive account details
    Get,
    /// Get account summary
    Summary,
    /// List tradeable instruments for account
    Instruments {
        /// Comma-separated list of instruments to query
        #[arg(long)]
        instruments: Option<String>,
    },
    /// Modify account configuration
    Configure {
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
    /// Poll account for changes since a transaction ID
    Changes {
        /// Transaction ID to poll changes since
        #[arg(long)]
        since_transaction_id: String,
    },
}

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    cmd: AccountCommand,
) -> OandaResult<()> {
    match cmd {
        AccountCommand::List => client.get("/v3/accounts", &[]).await,
        AccountCommand::Get => {
            let id = config.require_account_id()?;
            client.get(&format!("/v3/accounts/{id}"), &[]).await
        }
        AccountCommand::Summary => {
            let id = config.require_account_id()?;
            client.get(&format!("/v3/accounts/{id}/summary"), &[]).await
        }
        AccountCommand::Instruments { instruments } => {
            let id = config.require_account_id()?;
            let mut query = vec![];
            if let Some(ref i) = instruments {
                query.push(("instruments", i.as_str()));
            }
            client
                .get(&format!("/v3/accounts/{id}/instruments"), &query)
                .await
        }
        AccountCommand::Configure { body } => {
            config.require_mutation_allowed()?;
            let id = config.require_account_id()?;
            let body = read_body(body)?;
            client
                .patch(&format!("/v3/accounts/{id}/configuration"), body)
                .await
        }
        AccountCommand::Changes {
            since_transaction_id,
        } => {
            let id = config.require_account_id()?;
            client
                .get(
                    &format!("/v3/accounts/{id}/changes"),
                    &[("sinceTransactionID", since_transaction_id.as_str())],
                )
                .await
        }
    }
}
