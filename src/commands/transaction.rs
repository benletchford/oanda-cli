use clap::Subcommand;

use crate::client::OandaClient;
use crate::config::Config;

#[derive(Subcommand)]
pub enum TransactionCommand {
    /// Get transaction pages (time-based query)
    List {
        /// Start time filter
        #[arg(long)]
        from: Option<String>,
        /// End time filter
        #[arg(long)]
        to: Option<String>,
        /// Number of transactions per page
        #[arg(long)]
        page_size: Option<String>,
        /// Comma-separated transaction type filter
        #[arg(long, name = "type")]
        type_filter: Option<String>,
    },
    /// Get a single transaction by ID
    Get {
        /// Transaction ID
        transaction_id: String,
    },
    /// Get transactions in an ID range
    Idrange {
        /// Starting transaction ID
        #[arg(long)]
        from: String,
        /// Ending transaction ID
        #[arg(long)]
        to: String,
        /// Comma-separated transaction type filter
        #[arg(long, name = "type")]
        type_filter: Option<String>,
    },
    /// Get transactions since a specific ID
    Sinceid {
        /// Transaction ID to start from
        #[arg(long)]
        id: String,
        /// Comma-separated transaction type filter
        #[arg(long, name = "type")]
        type_filter: Option<String>,
    },
    /// Stream transactions in real-time
    Stream,
}

pub async fn execute(client: &OandaClient, config: &Config, cmd: TransactionCommand) -> Result<(), String> {
    let id = config.require_account_id()?;
    match cmd {
        TransactionCommand::List { from, to, page_size, type_filter } => {
            let mut query: Vec<(&str, &str)> = vec![];
            if let Some(ref v) = from { query.push(("from", v)); }
            if let Some(ref v) = to { query.push(("to", v)); }
            if let Some(ref v) = page_size { query.push(("pageSize", v)); }
            if let Some(ref v) = type_filter { query.push(("type", v)); }
            client.get(&format!("/v3/accounts/{id}/transactions"), &query).await
        }
        TransactionCommand::Get { transaction_id } => {
            client.get(&format!("/v3/accounts/{id}/transactions/{transaction_id}"), &[]).await
        }
        TransactionCommand::Idrange { from, to, type_filter } => {
            let mut query: Vec<(&str, &str)> = vec![("from", &from), ("to", &to)];
            if let Some(ref v) = type_filter { query.push(("type", v)); }
            client.get(&format!("/v3/accounts/{id}/transactions/idrange"), &query).await
        }
        TransactionCommand::Sinceid { id: since_id, type_filter } => {
            let mut query: Vec<(&str, &str)> = vec![("id", &since_id)];
            if let Some(ref v) = type_filter { query.push(("type", v)); }
            client.get(&format!("/v3/accounts/{id}/transactions/sinceid"), &query).await
        }
        TransactionCommand::Stream => {
            client.stream(&format!("/v3/accounts/{id}/transactions/stream"), &[]).await
        }
    }
}
