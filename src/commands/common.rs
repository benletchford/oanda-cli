use clap::ValueEnum;
use serde_json::{Map, Value, json};

use crate::client::{OandaError, OandaResult};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MarketTimeInForce {
    #[value(name = "FOK", alias = "fok")]
    Fok,
    #[value(name = "IOC", alias = "ioc")]
    Ioc,
}

impl MarketTimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fok => "FOK",
            Self::Ioc => "IOC",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PendingTimeInForce {
    #[value(name = "GTC", alias = "gtc")]
    Gtc,
    #[value(name = "GFD", alias = "gfd")]
    Gfd,
    #[value(name = "GTD", alias = "gtd")]
    Gtd,
}

impl PendingTimeInForce {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gtc => "GTC",
            Self::Gfd => "GFD",
            Self::Gtd => "GTD",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PositionFill {
    #[value(name = "DEFAULT", alias = "default")]
    Default,
    #[value(name = "OPEN_ONLY", alias = "open-only")]
    OpenOnly,
    #[value(name = "REDUCE_FIRST", alias = "reduce-first")]
    ReduceFirst,
    #[value(name = "REDUCE_ONLY", alias = "reduce-only")]
    ReduceOnly,
}

impl PositionFill {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::OpenOnly => "OPEN_ONLY",
            Self::ReduceFirst => "REDUCE_FIRST",
            Self::ReduceOnly => "REDUCE_ONLY",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TriggerCondition {
    #[value(name = "DEFAULT", alias = "default")]
    Default,
    #[value(name = "INVERSE", alias = "inverse")]
    Inverse,
    #[value(name = "BID", alias = "bid")]
    Bid,
    #[value(name = "ASK", alias = "ask")]
    Ask,
    #[value(name = "MID", alias = "mid")]
    Mid,
}

impl TriggerCondition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Inverse => "INVERSE",
            Self::Bid => "BID",
            Self::Ask => "ASK",
            Self::Mid => "MID",
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct ProtectionArgs {
    /// Take-profit price attached on fill
    #[arg(long)]
    pub take_profit: Option<String>,
    /// Stop-loss price attached on fill
    #[arg(long)]
    pub stop_loss: Option<String>,
    /// Trailing-stop distance attached on fill
    #[arg(long)]
    pub trailing_stop_distance: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct ClientExtensionsArgs {
    /// Client-defined order ID
    #[arg(long)]
    pub client_order_id: Option<String>,
    /// Client-defined order tag
    #[arg(long)]
    pub client_tag: Option<String>,
    /// Client-defined order comment
    #[arg(long)]
    pub client_comment: Option<String>,
}

pub fn validate_instrument(value: &str) -> OandaResult<()> {
    let mut parts = value.split('_');
    let first = parts.next().unwrap_or_default();
    let second = parts.next().unwrap_or_default();
    let valid_part = |part: &str| {
        (2..=12).contains(&part.len())
            && part
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    };
    if !valid_part(first) || !valid_part(second) || parts.next().is_some() {
        return Err(OandaError::Validation(format!(
            "Invalid instrument '{value}': expected an uppercase OANDA name such as EUR_USD"
        )));
    }
    Ok(())
}

pub fn validate_specifier(value: &str, label: &str) -> OandaResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'@' | b'-' | b'_' | b'.' | b':')
        })
    {
        return Err(OandaError::Validation(format!(
            "Invalid {label}: unsupported characters"
        )));
    }
    Ok(())
}

pub fn validate_decimal(value: &str, label: &str, allow_negative: bool) -> OandaResult<()> {
    if value.len() > 64 {
        return Err(OandaError::Validation(format!(
            "{label} exceeds the 64-character limit"
        )));
    }
    let unsigned = if let Some(unsigned) = value.strip_prefix('-') {
        if !allow_negative {
            return Err(OandaError::Validation(format!("{label} must be positive")));
        }
        unsigned
    } else {
        value
    };
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next();
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if !digits(whole)
        || fraction.is_some_and(|part| !digits(part))
        || parts.next().is_some()
        || unsigned.bytes().all(|byte| matches!(byte, b'0' | b'.'))
    {
        return Err(OandaError::Validation(format!(
            "{label} must be a non-zero decimal string"
        )));
    }
    Ok(())
}

pub fn add_protection(order: &mut Map<String, Value>, args: ProtectionArgs) -> OandaResult<()> {
    if let Some(price) = args.take_profit {
        validate_decimal(&price, "take-profit price", false)?;
        order.insert("takeProfitOnFill".into(), json!({ "price": price }));
    }
    if let Some(price) = args.stop_loss {
        validate_decimal(&price, "stop-loss price", false)?;
        order.insert("stopLossOnFill".into(), json!({ "price": price }));
    }
    if let Some(distance) = args.trailing_stop_distance {
        validate_decimal(&distance, "trailing-stop distance", false)?;
        order.insert(
            "trailingStopLossOnFill".into(),
            json!({ "distance": distance }),
        );
    }
    Ok(())
}

pub fn add_client_extensions(
    order: &mut Map<String, Value>,
    args: ClientExtensionsArgs,
) -> OandaResult<()> {
    let mut extensions = Map::new();
    add_limited_string(&mut extensions, "id", args.client_order_id, 128)?;
    add_limited_string(&mut extensions, "tag", args.client_tag, 128)?;
    add_limited_string(&mut extensions, "comment", args.client_comment, 128)?;
    if !extensions.is_empty() {
        order.insert("clientExtensions".into(), Value::Object(extensions));
    }
    Ok(())
}

fn add_limited_string(
    object: &mut Map<String, Value>,
    key: &str,
    value: Option<String>,
    limit: usize,
) -> OandaResult<()> {
    if let Some(value) = value {
        if value.is_empty() || value.len() > limit || value.contains(['\r', '\n']) {
            return Err(OandaError::Validation(format!(
                "Client {key} must contain 1 to {limit} characters on one line"
            )));
        }
        object.insert(key.into(), Value::String(value));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_instruments_and_decimals() {
        assert!(validate_instrument("EUR_USD").is_ok());
        assert!(validate_instrument("eur_usd").is_err());
        assert!(validate_decimal("-100.5", "units", true).is_ok());
        assert!(validate_decimal("0", "units", true).is_err());
        assert!(validate_decimal("1e3", "units", true).is_err());
    }
}
