use clap::{Subcommand, ValueEnum};
use serde::Serialize;

use crate::client::{OandaError, OandaResult};

#[derive(Debug, Clone, Copy, ValueEnum)]
#[allow(clippy::upper_case_acronyms)]
#[clap(rename_all = "UPPER")]
pub enum Instrument {
    AUDJPY,
    AUDUSD,
    EURAUD,
    EURCHF,
    EURGBP,
    EURJPY,
    EURUSD,
    GBPCHF,
    GBPJPY,
    GBPUSD,
    NZDUSD,
    USDCAD,
    USDCHF,
    USDJPY,
}

impl Instrument {
    #[allow(dead_code)]
    pub fn underscore(&self) -> &'static str {
        match self {
            Self::AUDJPY => "AUD_JPY",
            Self::AUDUSD => "AUD_USD",
            Self::EURAUD => "EUR_AUD",
            Self::EURCHF => "EUR_CHF",
            Self::EURGBP => "EUR_GBP",
            Self::EURJPY => "EUR_JPY",
            Self::EURUSD => "EUR_USD",
            Self::GBPCHF => "GBP_CHF",
            Self::GBPJPY => "GBP_JPY",
            Self::GBPUSD => "GBP_USD",
            Self::NZDUSD => "NZD_USD",
            Self::USDCAD => "USD_CAD",
            Self::USDCHF => "USD_CHF",
            Self::USDJPY => "USD_JPY",
        }
    }
}

impl std::fmt::Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AUDJPY => write!(f, "AUDJPY"),
            Self::AUDUSD => write!(f, "AUDUSD"),
            Self::EURAUD => write!(f, "EURAUD"),
            Self::EURCHF => write!(f, "EURCHF"),
            Self::EURGBP => write!(f, "EURGBP"),
            Self::EURJPY => write!(f, "EURJPY"),
            Self::EURUSD => write!(f, "EURUSD"),
            Self::GBPCHF => write!(f, "GBPCHF"),
            Self::GBPJPY => write!(f, "GBPJPY"),
            Self::GBPUSD => write!(f, "GBPUSD"),
            Self::NZDUSD => write!(f, "NZDUSD"),
            Self::USDCAD => write!(f, "USDCAD"),
            Self::USDCHF => write!(f, "USDCHF"),
            Self::USDJPY => write!(f, "USDJPY"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BookType {
    Order,
    Position,
}

impl BookType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BookType::Order => "ORDER",
            BookType::Position => "POSITION",
        }
    }
}

#[derive(Subcommand)]
pub enum LabsCommand {
    /// Fetch order/position book from OANDA Labs GraphQL API
    Book {
        /// Instrument name (e.g. EURUSD)
        instrument: Instrument,
        /// Book type: order or position
        #[arg(long)]
        book_type: BookType,
        /// Number of recent hours to query (default: 1)
        #[arg(long, default_value = "1")]
        recent_hours: i32,
    },
}

#[derive(Serialize)]
struct GraphQLRequest {
    #[serde(rename = "operationName")]
    operation_name: String,
    variables: GraphQLRequestVariables,
    query: String,
}

#[derive(Serialize)]
struct GraphQLRequestVariables {
    instrument: String,
    #[serde(rename = "bookType")]
    book_type: String,
    #[serde(rename = "recentHours")]
    recent_hours: i32,
}

const BOOK_QUERY: &str = "query GetOrderPositionBook($instrument: String!, $bookType: BookType!, $recentHours: Int) {\n  orderPositionBook(\n    instrument: $instrument\n    bookType: $bookType\n    recentHours: $recentHours\n  ) {\n    bucketWidth\n    price\n    time\n    buckets {\n      price\n      longCountPercent\n      shortCountPercent\n      __typename\n    }\n    __typename\n  }\n}";

pub async fn fetch_book(
    instrument: Instrument,
    book_type: BookType,
    recent_hours: i32,
) -> OandaResult<serde_json::Value> {
    if !(1..=168).contains(&recent_hours) {
        return Err(OandaError::Validation(
            "recent_hours must be between 1 and 168".into(),
        ));
    }
    let request = GraphQLRequest {
        operation_name: "GetOrderPositionBook".to_string(),
        variables: GraphQLRequestVariables {
            instrument: instrument.to_string(),
            book_type: book_type.as_str().to_string(),
            recent_hours,
        },
        query: BOOK_QUERY.to_string(),
    };

    let http = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(concat!("oanda-cli/", env!("CARGO_PKG_VERSION")))
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .build()
        .map_err(OandaError::Request)?;

    let resp: reqwest::Response = http
        .post("https://labs-api.oanda.com/graphql")
        .header("Accept", "*/*")
        .header("Content-Type", "application/json")
        .header("Sec-Fetch-Site", "same-site")
        .header("Accept-Language", "en-AU,en;q=0.9")
        .header("Sec-Fetch-Mode", "cors")
        .header("Origin", "https://www.oanda.com")
        .header("Referer", "https://www.oanda.com/")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.4 Safari/605.1.15")
        .header("Sec-Fetch-Dest", "empty")
        .header("Priority", "u=3, i")
        .json(&request)
        .send()
        .await
        .map_err(OandaError::Request)?;

    let status = resp.status();
    let body = resp
        .json::<serde_json::Value>()
        .await
        .map_err(OandaError::Decode)?;

    if !status.is_success() {
        return Err(OandaError::Api { status, body });
    }

    if body
        .get("errors")
        .is_some_and(|errors| errors.as_array().is_none_or(|errors| !errors.is_empty()))
    {
        return Err(OandaError::Response(format!(
            "OANDA Labs GraphQL returned errors: {}",
            body["errors"]
        )));
    }

    Ok(body)
}

pub async fn execute(pretty: bool, cmd: LabsCommand) -> OandaResult<()> {
    match cmd {
        LabsCommand::Book {
            instrument,
            book_type,
            recent_hours,
        } => {
            let body = fetch_book(instrument, book_type, recent_hours).await?;

            let output = if pretty {
                serde_json::to_string_pretty(&body)
            } else {
                serde_json::to_string(&body)
            }
            .map_err(OandaError::Json)?;
            println!("{output}");
            Ok(())
        }
    }
}
