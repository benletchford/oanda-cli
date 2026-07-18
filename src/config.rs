use std::{env, fmt, time::Duration};

use serde::Serialize;

use crate::client::{OandaError, OandaResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Practice,
    Live,
}

impl Environment {
    pub fn base_url(&self) -> &str {
        match self {
            Environment::Practice => "https://api-fxpractice.oanda.com",
            Environment::Live => "https://api-fxtrade.oanda.com",
        }
    }

    pub fn stream_url(&self) -> &str {
        match self {
            Environment::Practice => "https://stream-fxpractice.oanda.com",
            Environment::Live => "https://stream-fxtrade.oanda.com",
        }
    }

    pub fn parse(value: &str) -> OandaResult<Self> {
        match value {
            "practice" => Ok(Self::Practice),
            "live" => Ok(Self::Live),
            other => Err(OandaError::Validation(format!(
                "Invalid environment '{other}': expected 'practice' or 'live'"
            ))),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Practice => write!(f, "practice"),
            Environment::Live => write!(f, "live"),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub token: Option<String>,
    pub account_id: Option<String>,
    pub environment: Environment,
    pub datetime_format: Option<String>,
    pub pretty: bool,
    pub dry_run: bool,
    pub confirm_live: bool,
    pub request_timeout: Duration,
    pub connect_timeout: Duration,
    environment_explicit: bool,
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("token", &self.token.as_ref().map(|_| "[REDACTED]"))
            .field(
                "account_id",
                &self.account_id.as_ref().map(|_| "[REDACTED]"),
            )
            .field("environment", &self.environment)
            .field("datetime_format", &self.datetime_format)
            .field("pretty", &self.pretty)
            .field("dry_run", &self.dry_run)
            .field("confirm_live", &self.confirm_live)
            .field("request_timeout", &self.request_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("environment_explicit", &self.environment_explicit)
            .finish()
    }
}

impl Config {
    pub fn new(token: impl Into<String>, account_id: impl Into<String>) -> Self {
        Config {
            token: Some(token.into()),
            account_id: Some(account_id.into()),
            environment: Environment::Practice,
            datetime_format: None,
            pretty: false,
            dry_run: false,
            confirm_live: false,
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            environment_explicit: false,
        }
    }

    pub fn from_env() -> OandaResult<Self> {
        Self::from_args(None, None, None, None, false)
    }

    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self.environment_explicit = true;
        self
    }

    pub fn with_datetime_format(mut self, datetime_format: impl Into<String>) -> Self {
        self.datetime_format = Some(datetime_format.into());
        self
    }

    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_confirm_live(mut self, confirm_live: bool) -> Self {
        self.confirm_live = confirm_live;
        self
    }

    pub fn with_timeouts(
        mut self,
        request_timeout_secs: Option<u64>,
        connect_timeout_secs: Option<u64>,
    ) -> OandaResult<Self> {
        let request_timeout_secs = timeout_value(
            request_timeout_secs,
            "OANDA_REQUEST_TIMEOUT_SECS",
            self.request_timeout.as_secs(),
        )?;
        let connect_timeout_secs = timeout_value(
            connect_timeout_secs,
            "OANDA_CONNECT_TIMEOUT_SECS",
            self.connect_timeout.as_secs(),
        )?;
        if request_timeout_secs == 0 || connect_timeout_secs == 0 {
            return Err(OandaError::Validation(
                "Timeouts must be greater than zero seconds".into(),
            ));
        }
        self.request_timeout = Duration::from_secs(request_timeout_secs);
        self.connect_timeout = Duration::from_secs(connect_timeout_secs);
        Ok(self)
    }

    pub fn from_args(
        token: Option<String>,
        account_id: Option<String>,
        environment: Option<String>,
        datetime_format: Option<String>,
        pretty: bool,
    ) -> OandaResult<Self> {
        let token = nonempty(token.or_else(|| env::var("OANDA_ACCESS_TOKEN").ok()));

        let account_id = nonempty(account_id.or_else(|| env::var("OANDA_ACCOUNT_ID").ok()));
        if let Some(account_id) = &account_id {
            validate_account_id(account_id)?;
        }

        let env_from_var = nonempty(env::var("OANDA_ENVIRONMENT").ok());
        let environment_explicit = environment.is_some() || env_from_var.is_some();
        let env_str = nonempty(environment).or(env_from_var);
        let environment = match env_str {
            Some(value) => Environment::parse(&value)?,
            None => Environment::Practice,
        };

        let datetime_format =
            nonempty(datetime_format.or_else(|| env::var("OANDA_DATETIME_FORMAT").ok()));
        if let Some(value) = &datetime_format {
            if value != "RFC3339" && value != "UNIX" {
                return Err(OandaError::Validation(format!(
                    "Invalid datetime format '{value}': expected 'RFC3339' or 'UNIX'"
                )));
            }
        }

        Ok(Config {
            token,
            account_id,
            environment,
            datetime_format,
            pretty,
            dry_run: false,
            confirm_live: false,
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            environment_explicit,
        })
    }

    pub fn require_token(&self) -> OandaResult<&str> {
        self.token.as_deref().ok_or_else(|| {
            OandaError::Config(
                "Access token required: pass --token or set OANDA_ACCESS_TOKEN".into(),
            )
        })
    }

    pub fn require_account_id(&self) -> OandaResult<&str> {
        self.account_id.as_deref().ok_or_else(|| {
            OandaError::Config(
                "Account ID required: pass --account-id or set OANDA_ACCOUNT_ID".into(),
            )
        })
    }

    pub fn require_mutation_allowed(&self) -> OandaResult<()> {
        if !self.environment_explicit {
            return Err(OandaError::Config(
                "Mutations require an explicit environment: pass --environment or set OANDA_ENVIRONMENT"
                    .into(),
            ));
        }
        if self.environment == Environment::Live && !self.confirm_live {
            return Err(OandaError::Config(
                "Live mutations require --confirm-live".into(),
            ));
        }
        Ok(())
    }
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn timeout_value(value: Option<u64>, variable: &str, default: u64) -> OandaResult<u64> {
    match value {
        Some(value) => Ok(value),
        None => match env::var(variable) {
            Ok(value) => value.parse::<u64>().map_err(|_| {
                OandaError::Validation(format!("{variable} must be a whole number of seconds"))
            }),
            Err(_) => Ok(default),
        },
    }
}

pub(crate) fn validate_account_id(value: &str) -> OandaResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(OandaError::Validation(
            "Account ID contains unsupported characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_environments() {
        assert_eq!(
            Environment::parse("practice").unwrap(),
            Environment::Practice
        );
        assert_eq!(Environment::parse("live").unwrap(), Environment::Live);
        assert!(Environment::parse("demo").is_err());
    }

    #[test]
    fn live_mutations_need_confirmation() {
        let config = Config::new("token", "101-001-123-001").with_environment(Environment::Live);
        assert!(config.require_mutation_allowed().is_err());
        assert!(
            config
                .with_confirm_live(true)
                .require_mutation_allowed()
                .is_ok()
        );
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let output = format!("{:?}", Config::new("super-secret", "101-001-123-001"));
        assert!(!output.contains("super-secret"));
        assert!(!output.contains("101-001-123-001"));
        assert!(output.contains("[REDACTED]"));
    }
}
