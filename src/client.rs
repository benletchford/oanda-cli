use futures_util::StreamExt;
use reqwest::{Client, Method, StatusCode};
use serde::Serialize;
use serde_json::{Value, json};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

use crate::config::{Config, Environment, validate_account_id};

const MAX_BODY_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
const MAX_STREAM_LINE_BYTES: usize = 1024 * 1024;

pub type OandaResult<T> = Result<T, OandaError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Configuration,
    Validation,
    Authentication,
    Network,
    Timeout,
    Api,
    Response,
    Io,
}

#[derive(Debug)]
pub enum OandaError {
    Config(String),
    Validation(String),
    Request(reqwest::Error),
    Decode(reqwest::Error),
    Json(serde_json::Error),
    Response(String),
    Io(std::io::Error),
    Api { status: StatusCode, body: Value },
}

impl OandaError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Config(_) => ErrorKind::Configuration,
            Self::Validation(_) => ErrorKind::Validation,
            Self::Request(error) | Self::Decode(error) if error.is_timeout() => ErrorKind::Timeout,
            Self::Request(_) => ErrorKind::Network,
            Self::Decode(_) | Self::Json(_) | Self::Response(_) => ErrorKind::Response,
            Self::Io(_) => ErrorKind::Io,
            Self::Api { status, .. }
                if *status == StatusCode::UNAUTHORIZED || *status == StatusCode::FORBIDDEN =>
            {
                ErrorKind::Authentication
            }
            Self::Api { .. } => ErrorKind::Api,
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.kind() {
            ErrorKind::Validation => 2,
            ErrorKind::Configuration | ErrorKind::Authentication => 3,
            ErrorKind::Network | ErrorKind::Timeout => 4,
            ErrorKind::Api => 5,
            ErrorKind::Response | ErrorKind::Io => 6,
        }
    }

    pub fn structured(&self) -> Value {
        let mut error = json!({
            "kind": self.kind(),
            "message": self.to_string(),
            "exitCode": self.exit_code(),
        });
        if let Self::Api { status, body } = self {
            error["status"] = json!(status.as_u16());
            error["details"] = redact_json(body.clone());
        }
        json!({ "error": error })
    }
}

impl fmt::Display for OandaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) | Self::Validation(error) | Self::Response(error) => {
                write!(f, "{error}")
            }
            Self::Request(error) => write!(f, "Request failed: {error}"),
            Self::Decode(error) => write!(f, "Failed to read response: {error}"),
            Self::Json(error) => write!(f, "Failed to parse JSON response: {error}"),
            Self::Io(error) => write!(f, "I/O failed: {error}"),
            Self::Api { status, body } => {
                let body = serde_json::to_string(&redact_json(body.clone()))
                    .unwrap_or_else(|_| "<unavailable>".into());
                write!(f, "OANDA API returned {status}: {body}")
            }
        }
    }
}

impl Error for OandaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) | Self::Decode(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Config(_) | Self::Validation(_) | Self::Response(_) | Self::Api { .. } => None,
        }
    }
}

pub struct OandaClient {
    http: Client,
    base_url: String,
    stream_url: String,
    token: Option<String>,
    account_id: Option<String>,
    datetime_format: Option<String>,
    environment: Environment,
    request_timeout: std::time::Duration,
    pretty: bool,
    dry_run: bool,
}

impl OandaClient {
    pub fn new(config: &Config) -> OandaResult<Self> {
        if let Some(account_id) = &config.account_id {
            validate_account_id(account_id)?;
        }
        if !config.dry_run {
            let token = config.require_token()?;
            if token.trim().is_empty() {
                return Err(OandaError::Config("Access token cannot be empty".into()));
            }
        }
        let http = Client::builder()
            .connect_timeout(config.connect_timeout)
            .user_agent(concat!("oanda-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(OandaError::Request)?;
        Ok(Self {
            http,
            base_url: config.environment.base_url().into(),
            stream_url: config.environment.stream_url().into(),
            token: config.token.clone(),
            account_id: config.account_id.clone(),
            datetime_format: config.datetime_format.clone(),
            environment: config.environment,
            request_timeout: config.request_timeout,
            pretty: config.pretty,
            dry_run: config.dry_run,
        })
    }

    /// Override service URLs for testing or a trusted OANDA-compatible proxy.
    ///
    /// The access token will be sent to these URLs. Only use trusted HTTPS endpoints.
    pub fn with_base_urls(
        config: &Config,
        base_url: impl Into<String>,
        stream_url: impl Into<String>,
    ) -> OandaResult<Self> {
        let mut client = Self::new(config)?;
        client.base_url = normalize_base_url(base_url.into())?;
        client.stream_url = normalize_base_url(stream_url.into())?;
        Ok(client)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn stream_url(&self) -> &str {
        &self.stream_url
    }

    pub fn default_account_id(&self) -> Option<&str> {
        self.account_id.as_deref()
    }

    pub(crate) fn require_account_id(&self) -> OandaResult<&str> {
        self.default_account_id()
            .ok_or_else(|| OandaError::Config("Account ID required: set it on Config".into()))
    }

    fn request(&self, method: Method, path: &str) -> OandaResult<reqwest::RequestBuilder> {
        validate_path(path)?;
        let token = self
            .token
            .as_deref()
            .ok_or_else(|| OandaError::Config("Access token required".into()))?;
        let url = format!("{}{path}", self.base_url);
        let mut request = self
            .http
            .request(method, &url)
            .bearer_auth(token)
            .timeout(self.request_timeout);
        if let Some(format) = &self.datetime_format {
            request = request.header("Accept-Datetime-Format", format.as_str());
        }
        Ok(request)
    }

    fn stream_request(&self, path: &str) -> OandaResult<reqwest::RequestBuilder> {
        validate_path(path)?;
        let token = self
            .token
            .as_deref()
            .ok_or_else(|| OandaError::Config("Access token required".into()))?;
        let url = format!("{}{path}", self.stream_url);
        let mut request = self.http.get(&url).bearer_auth(token);
        if let Some(format) = &self.datetime_format {
            request = request.header("Accept-Datetime-Format", format.as_str());
        }
        Ok(request)
    }

    pub fn print_json(&self, value: &Value) -> OandaResult<()> {
        let output = if self.pretty {
            serde_json::to_string_pretty(value)
        } else {
            serde_json::to_string(value)
        }
        .map_err(OandaError::Json)?;
        match writeln!(std::io::stdout(), "{output}") {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
            Err(error) => Err(OandaError::Io(error)),
        }
    }

    async fn response_json(response: reqwest::Response) -> OandaResult<Value> {
        let status = response.status();
        let body = Self::read_json_or_text(response, status.is_success()).await?;
        if !status.is_success() {
            return Err(OandaError::Api { status, body });
        }
        Ok(body)
    }

    async fn read_json_or_text(
        response: reqwest::Response,
        require_json: bool,
    ) -> OandaResult<Value> {
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(OandaError::Response(format!(
                "Response exceeds the {MAX_RESPONSE_BYTES}-byte limit"
            )));
        }
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(OandaError::Decode)?;
            if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
                return Err(OandaError::Response(format!(
                    "Response exceeds the {MAX_RESPONSE_BYTES}-byte limit"
                )));
            }
            bytes.extend_from_slice(&chunk);
        }
        match serde_json::from_slice(&bytes) {
            Ok(body) => Ok(body),
            Err(error) if require_json => Err(OandaError::Json(error)),
            Err(_) => Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())),
        }
    }

    pub async fn request_json(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> OandaResult<Value> {
        validate_path(path)?;
        if self.dry_run && method != Method::GET && method != Method::HEAD {
            return Ok(json!({
                "dryRun": true,
                "environment": self.environment,
                "mutation": true,
                "method": method.as_str(),
                "endpoint": path,
                "query": query.iter().map(|(key, value)| json!({"name": key, "value": value})).collect::<Vec<_>>(),
                "body": body.map(redact_json),
            }));
        }

        let mut request = self.request(method, path)?.query(query);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(OandaError::Request)?;
        Self::response_json(response).await
    }

    pub async fn get_json(&self, path: &str, query: &[(&str, &str)]) -> OandaResult<Value> {
        self.request_json(Method::GET, path, query, None).await
    }

    pub async fn post_json(&self, path: &str, body: Value) -> OandaResult<Value> {
        self.request_json(Method::POST, path, &[], Some(body)).await
    }

    pub async fn put_json(&self, path: &str, body: Option<Value>) -> OandaResult<Value> {
        self.request_json(Method::PUT, path, &[], body).await
    }

    pub async fn patch_json(&self, path: &str, body: Value) -> OandaResult<Value> {
        self.request_json(Method::PATCH, path, &[], Some(body))
            .await
    }

    pub async fn stream_response(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> OandaResult<reqwest::Response> {
        let response = self
            .stream_request(path)?
            .query(query)
            .send()
            .await
            .map_err(OandaError::Request)?;
        let status = response.status();
        if !status.is_success() {
            let body = Self::read_json_or_text(response, false).await?;
            return Err(OandaError::Api { status, body });
        }
        Ok(response)
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> OandaResult<()> {
        let body = self.get_json(path, query).await?;
        self.print_json(&body)
    }

    pub async fn post(&self, path: &str, body: Value) -> OandaResult<()> {
        let body = self.post_json(path, body).await?;
        self.print_json(&body)
    }

    pub async fn put(&self, path: &str, body: Option<Value>) -> OandaResult<()> {
        let body = self.put_json(path, body).await?;
        self.print_json(&body)
    }

    pub async fn patch(&self, path: &str, body: Value) -> OandaResult<()> {
        let body = self.patch_json(path, body).await?;
        self.print_json(&body)
    }

    pub async fn stream(&self, path: &str, query: &[(&str, &str)]) -> OandaResult<()> {
        let response = self.stream_response(path, query).await?;
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut stdout = std::io::stdout();
        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                result = &mut shutdown => {
                    result.map_err(OandaError::Io)?;
                    break;
                }
                chunk = stream.next() => {
                    let Some(chunk) = chunk else { break };
                    let chunk = chunk.map_err(OandaError::Request)?;
                    buffer.extend_from_slice(&chunk);
                    while let Some(position) = buffer.iter().position(|byte| *byte == b'\n') {
                        if position > MAX_STREAM_LINE_BYTES {
                            return Err(OandaError::Response(format!(
                                "Stream line exceeds the {MAX_STREAM_LINE_BYTES}-byte limit"
                            )));
                        }
                        let line: Vec<u8> = buffer.drain(..=position).collect();
                        if let Err(error) = write_stream_line(&mut stdout, &line, self.pretty) {
                            if is_broken_pipe(&error) {
                                return Ok(());
                            }
                            return Err(error);
                        }
                    }
                    if buffer.len() > MAX_STREAM_LINE_BYTES {
                        return Err(OandaError::Response(format!(
                            "Stream line exceeds the {MAX_STREAM_LINE_BYTES}-byte limit"
                        )));
                    }
                }
            }
        }
        if !buffer.is_empty() {
            if let Err(error) = write_stream_line(&mut stdout, &buffer, self.pretty) {
                if is_broken_pipe(&error) {
                    return Ok(());
                }
                return Err(error);
            }
        }
        match stdout.flush() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
            Err(error) => Err(OandaError::Io(error)),
        }
    }
}

pub fn read_body(body: Option<String>) -> OandaResult<Value> {
    let bytes = match body {
        Some(body) => body.into_bytes(),
        None => {
            let mut bytes = Vec::new();
            std::io::stdin()
                .lock()
                .take((MAX_BODY_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(OandaError::Io)?;
            bytes
        }
    };
    if bytes.len() > MAX_BODY_BYTES {
        return Err(OandaError::Validation(format!(
            "JSON body exceeds the {MAX_BODY_BYTES}-byte limit"
        )));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| OandaError::Validation(format!("Invalid JSON body: {error}")))?;
    if !value.is_object() {
        return Err(OandaError::Validation(
            "JSON request body must be an object".into(),
        ));
    }
    Ok(value)
}

pub fn redact_json(mut value: Value) -> Value {
    match &mut value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase();
                if normalized.contains("token")
                    || normalized.contains("password")
                    || normalized.contains("secret")
                    || normalized == "authorization"
                {
                    *child = Value::String("[REDACTED]".into());
                } else {
                    *child = redact_json(child.take());
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                *item = redact_json(item.take());
            }
        }
        _ => {}
    }
    value
}

fn normalize_base_url(value: String) -> OandaResult<String> {
    let value = value.trim_end_matches('/').to_owned();
    let url = reqwest::Url::parse(&value)
        .map_err(|error| OandaError::Validation(format!("Invalid base URL: {error}")))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(OandaError::Validation(
            "Base URL must use HTTP or HTTPS and include a host".into(),
        ));
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(OandaError::Validation(
            "Base URL cannot include credentials, a query, or a fragment".into(),
        ));
    }
    Ok(value)
}

fn validate_path(path: &str) -> OandaResult<()> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains("..")
        || path.bytes().any(|byte| byte == b'\r' || byte == b'\n')
    {
        return Err(OandaError::Validation("Invalid API path".into()));
    }
    Ok(())
}

fn write_stream_line(stdout: &mut std::io::Stdout, bytes: &[u8], pretty: bool) -> OandaResult<()> {
    let line = String::from_utf8_lossy(bytes);
    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }
    if pretty {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => serde_json::to_string_pretty(&value)
                .map_err(OandaError::Json)
                .and_then(|value| writeln!(stdout, "{value}").map_err(OandaError::Io)),
            Err(_) => writeln!(stdout, "{line}").map_err(OandaError::Io),
        }
    } else {
        writeln!(stdout, "{line}").map_err(OandaError::Io)
    }
}

fn is_broken_pipe(error: &OandaError) -> bool {
    matches!(error, OandaError::Io(error) if error.kind() == std::io::ErrorKind::BrokenPipe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_nested_secrets() {
        let value = json!({"accessToken": "secret", "nested": {"password": "secret", "ok": 1}});
        assert_eq!(
            redact_json(value),
            json!({"accessToken": "[REDACTED]", "nested": {"password": "[REDACTED]", "ok": 1}})
        );
    }

    #[test]
    fn error_kinds_have_stable_exit_codes() {
        let error = OandaError::Validation("bad input".into());
        assert_eq!(error.kind(), ErrorKind::Validation);
        assert_eq!(error.exit_code(), 2);
        assert_eq!(error.structured()["error"]["kind"], "validation");
    }

    #[test]
    fn base_url_rejects_embedded_credentials() {
        assert!(normalize_base_url("https://token@example.com".into()).is_err());
        assert!(normalize_base_url("https://example.com?token=x".into()).is_err());
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8080/".into()).unwrap(),
            "http://127.0.0.1:8080"
        );
    }
}
