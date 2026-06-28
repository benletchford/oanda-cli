use clap::Subcommand;

use crate::client::{OandaClient, read_body};
use crate::config::Config;

#[derive(Subcommand)]
pub enum OrderCommand {
    /// Create an order
    Create {
        /// JSON request body (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,
    },
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
    /// Replace an order with new specification
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

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    cmd: OrderCommand,
) -> Result<(), String> {
    let id = config.require_account_id()?;
    match cmd {
        OrderCommand::Create { body } => {
            let body = read_body(body)?;
            client
                .post(&format!("/v3/accounts/{id}/orders"), body)
                .await
        }
        OrderCommand::List {
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
                .get(&format!("/v3/accounts/{id}/orders"), &query)
                .await
        }
        OrderCommand::Pending => {
            client
                .get(&format!("/v3/accounts/{id}/pendingOrders"), &[])
                .await
        }
        OrderCommand::Get { order_specifier } => {
            client
                .get(&format!("/v3/accounts/{id}/orders/{order_specifier}"), &[])
                .await
        }
        OrderCommand::Replace {
            order_specifier,
            body,
        } => {
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{id}/orders/{order_specifier}"),
                    Some(body),
                )
                .await
        }
        OrderCommand::Cancel { order_specifier } => {
            client
                .put(
                    &format!("/v3/accounts/{id}/orders/{order_specifier}/cancel"),
                    None,
                )
                .await
        }
        OrderCommand::ClientExtensions {
            order_specifier,
            body,
        } => {
            let body = read_body(body)?;
            client
                .put(
                    &format!("/v3/accounts/{id}/orders/{order_specifier}/clientExtensions"),
                    Some(body),
                )
                .await
        }
    }
}
