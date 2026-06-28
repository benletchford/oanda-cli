use clap::Subcommand;

use crate::client::OandaClient;
use crate::config::Config;

#[derive(Subcommand)]
pub enum PricingCommand {
    /// Get current pricing for instruments
    Get {
        /// Comma-separated instrument names (e.g. EUR_USD,USD_JPY)
        #[arg(long)]
        instruments: String,
        /// Only return prices since this time
        #[arg(long)]
        since: Option<String>,
        /// Include units available for trading
        #[arg(long)]
        include_units_available: bool,
        /// Include home conversion factors
        #[arg(long)]
        include_home_conversions: bool,
    },
    /// Stream live prices for instruments
    Stream {
        /// Comma-separated instrument names
        #[arg(long)]
        instruments: String,
        /// Include initial snapshot
        #[arg(long)]
        snapshot: bool,
        /// Include home conversion factors
        #[arg(long)]
        include_home_conversions: bool,
    },
    /// Get latest completed candlesticks
    CandlesLatest {
        /// Comma-separated candle specifications (e.g. EUR_USD:S5:MBA)
        #[arg(long)]
        candle_specifications: String,
        /// Number of units for volume calculation
        #[arg(long)]
        units: Option<String>,
        /// Enable smoothing
        #[arg(long)]
        smooth: bool,
        /// Hour of day for daily alignment (0-23)
        #[arg(long)]
        daily_alignment: Option<String>,
        /// Timezone for daily alignment
        #[arg(long)]
        alignment_timezone: Option<String>,
        /// Day of week for weekly alignment
        #[arg(long)]
        weekly_alignment: Option<String>,
    },
    /// Get candlestick data for an instrument (account-scoped)
    Candles {
        /// Instrument name (e.g. EUR_USD)
        instrument: String,
        /// Bid/Ask/Mid pricing
        #[arg(long)]
        price: Option<String>,
        /// Candlestick granularity
        #[arg(long)]
        granularity: Option<String>,
        /// Number of candles to return
        #[arg(long)]
        count: Option<String>,
        /// Start time
        #[arg(long)]
        from: Option<String>,
        /// End time
        #[arg(long)]
        to: Option<String>,
        /// Enable smoothing
        #[arg(long)]
        smooth: bool,
        /// Include first candle
        #[arg(long)]
        include_first: Option<String>,
        /// Hour of day for daily alignment
        #[arg(long)]
        daily_alignment: Option<String>,
        /// Timezone for daily alignment
        #[arg(long)]
        alignment_timezone: Option<String>,
        /// Day of week for weekly alignment
        #[arg(long)]
        weekly_alignment: Option<String>,
        /// Number of units for volume calculation
        #[arg(long)]
        units: Option<String>,
    },
}

pub async fn execute(
    client: &OandaClient,
    config: &Config,
    cmd: PricingCommand,
) -> Result<(), String> {
    let id = config.require_account_id()?;
    match cmd {
        PricingCommand::Get {
            instruments,
            since,
            include_units_available,
            include_home_conversions,
        } => {
            let mut query: Vec<(&str, &str)> = vec![("instruments", &instruments)];
            if let Some(ref v) = since {
                query.push(("since", v));
            }
            if include_units_available {
                query.push(("includeUnitsAvailable", "true"));
            }
            if include_home_conversions {
                query.push(("includeHomeConversions", "true"));
            }
            client
                .get(&format!("/v3/accounts/{id}/pricing"), &query)
                .await
        }
        PricingCommand::Stream {
            instruments,
            snapshot,
            include_home_conversions,
        } => {
            let mut query: Vec<(&str, &str)> = vec![("instruments", &instruments)];
            if snapshot {
                query.push(("snapshot", "true"));
            }
            if include_home_conversions {
                query.push(("includeHomeConversions", "true"));
            }
            client
                .stream(&format!("/v3/accounts/{id}/pricing/stream"), &query)
                .await
        }
        PricingCommand::CandlesLatest {
            candle_specifications,
            units,
            smooth,
            daily_alignment,
            alignment_timezone,
            weekly_alignment,
        } => {
            let mut query: Vec<(&str, &str)> =
                vec![("candleSpecifications", &candle_specifications)];
            if let Some(ref v) = units {
                query.push(("units", v));
            }
            if smooth {
                query.push(("smooth", "true"));
            }
            if let Some(ref v) = daily_alignment {
                query.push(("dailyAlignment", v));
            }
            if let Some(ref v) = alignment_timezone {
                query.push(("alignmentTimezone", v));
            }
            if let Some(ref v) = weekly_alignment {
                query.push(("weeklyAlignment", v));
            }
            client
                .get(&format!("/v3/accounts/{id}/candles/latest"), &query)
                .await
        }
        PricingCommand::Candles {
            instrument,
            price,
            granularity,
            count,
            from,
            to,
            smooth,
            include_first,
            daily_alignment,
            alignment_timezone,
            weekly_alignment,
            units,
        } => {
            let mut query: Vec<(&str, &str)> = vec![];
            if let Some(ref v) = price {
                query.push(("price", v));
            }
            if let Some(ref v) = granularity {
                query.push(("granularity", v));
            }
            if let Some(ref v) = count {
                query.push(("count", v));
            }
            if let Some(ref v) = from {
                query.push(("from", v));
            }
            if let Some(ref v) = to {
                query.push(("to", v));
            }
            if smooth {
                query.push(("smooth", "true"));
            }
            if let Some(ref v) = include_first {
                query.push(("includeFirst", v));
            }
            if let Some(ref v) = daily_alignment {
                query.push(("dailyAlignment", v));
            }
            if let Some(ref v) = alignment_timezone {
                query.push(("alignmentTimezone", v));
            }
            if let Some(ref v) = weekly_alignment {
                query.push(("weeklyAlignment", v));
            }
            if let Some(ref v) = units {
                query.push(("units", v));
            }
            client
                .get(
                    &format!("/v3/accounts/{id}/instruments/{instrument}/candles"),
                    &query,
                )
                .await
        }
    }
}
