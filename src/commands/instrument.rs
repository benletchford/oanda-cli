use clap::Subcommand;

use crate::client::OandaClient;

#[derive(Subcommand)]
pub enum InstrumentCommand {
    /// Fetch candlestick data for an instrument
    Candles {
        /// Instrument name (e.g. EUR_USD)
        instrument: String,
        /// Bid/Ask/Mid pricing (e.g. "M", "BA", "MBA")
        #[arg(long)]
        price: Option<String>,
        /// Candlestick granularity (e.g. S5, M1, H1, D)
        #[arg(long)]
        granularity: Option<String>,
        /// Number of candles to return (max 5000)
        #[arg(long)]
        count: Option<String>,
        /// Start time (RFC3339 or Unix)
        #[arg(long)]
        from: Option<String>,
        /// End time (RFC3339 or Unix)
        #[arg(long)]
        to: Option<String>,
        /// Enable smoothing
        #[arg(long)]
        smooth: bool,
        /// Include first candle
        #[arg(long)]
        include_first: Option<String>,
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
}

pub async fn execute(client: &OandaClient, cmd: InstrumentCommand) -> Result<(), String> {
    match cmd {
        InstrumentCommand::Candles {
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

            client
                .get(&format!("/v3/instruments/{instrument}/candles"), &query)
                .await
        }
    }
}
