use std::{env, fmt};

#[derive(Debug, Clone)]
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
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Practice => write!(f, "practice"),
            Environment::Live => write!(f, "live"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub token: Option<String>,
    pub account_id: Option<String>,
    pub environment: Environment,
    pub datetime_format: Option<String>,
    pub pretty: bool,
}

impl Config {
    pub fn new(token: impl Into<String>, account_id: impl Into<String>) -> Self {
        Config {
            token: Some(token.into()),
            account_id: Some(account_id.into()),
            environment: Environment::Practice,
            datetime_format: None,
            pretty: false,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        Self::from_args(None, None, None, None, false)
    }

    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
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

    pub fn from_args(
        token: Option<String>,
        account_id: Option<String>,
        environment: Option<String>,
        datetime_format: Option<String>,
        pretty: bool,
    ) -> Result<Self, String> {
        let token = token.or_else(|| env::var("OANDA_TOKEN").ok());

        let account_id = account_id.or_else(|| env::var("OANDA_ACCOUNT_ID").ok());

        let env_str = environment.or_else(|| env::var("OANDA_ENVIRONMENT").ok());
        let environment = match env_str.as_deref() {
            Some("live") => Environment::Live,
            Some("practice") | None => Environment::Practice,
            Some(other) => {
                return Err(format!(
                    "Invalid environment: {other}. Use 'practice' or 'live'"
                ));
            }
        };

        let datetime_format = datetime_format.or_else(|| env::var("OANDA_DATETIME_FORMAT").ok());

        Ok(Config {
            token,
            account_id,
            environment,
            datetime_format,
            pretty,
        })
    }

    pub fn require_token(&self) -> Result<&str, String> {
        self.token
            .as_deref()
            .ok_or_else(|| "Token required: pass --token or set OANDA_TOKEN".into())
    }

    pub fn require_account_id(&self) -> Result<&str, String> {
        self.account_id
            .as_deref()
            .ok_or_else(|| "Account ID required: pass --account-id or set OANDA_ACCOUNT_ID".into())
    }
}
