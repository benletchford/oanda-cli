use futures_util::StreamExt;
use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::io::Write;

use crate::config::Config;

pub type OandaResult<T> = Result<T, OandaError>;

#[derive(Debug)]
pub enum OandaError {
    Config(String),
    Request(reqwest::Error),
    Decode(reqwest::Error),
    Json(serde_json::Error),
    Api {
        status: reqwest::StatusCode,
        body: serde_json::Value,
    },
}

impl fmt::Display for OandaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OandaError::Config(err) => write!(f, "{err}"),
            OandaError::Request(err) => write!(f, "Request failed: {err}"),
            OandaError::Decode(err) => write!(f, "Failed to read response: {err}"),
            OandaError::Json(err) => write!(f, "Failed to parse JSON response: {err}"),
            OandaError::Api { status, body } => {
                let body = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
                write!(f, "OANDA API returned {status}: {body}")
            }
        }
    }
}

impl Error for OandaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OandaError::Config(_) | OandaError::Api { .. } => None,
            OandaError::Request(err) | OandaError::Decode(err) => Some(err),
            OandaError::Json(err) => Some(err),
        }
    }
}

pub struct OandaClient {
    http: Client,
    base_url: String,
    stream_url: String,
    token: String,
    account_id: Option<String>,
    datetime_format: Option<String>,
    pretty: bool,
}

impl OandaClient {
    pub fn new(config: &Config) -> OandaResult<Self> {
        let token = config
            .require_token()
            .map_err(OandaError::Config)?
            .to_string();
        Ok(OandaClient {
            http: Client::new(),
            base_url: config.environment.base_url().to_string(),
            stream_url: config.environment.stream_url().to_string(),
            token,
            account_id: config.account_id.clone(),
            datetime_format: config.datetime_format.clone(),
            pretty: config.pretty,
        })
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

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        let mut req = self.http.request(method, &url).bearer_auth(&self.token);
        if let Some(fmt) = &self.datetime_format {
            req = req.header("Accept-Datetime-Format", fmt.as_str());
        }
        req
    }

    fn stream_request(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{path}", self.stream_url);
        let mut req = self.http.get(&url).bearer_auth(&self.token);
        if let Some(fmt) = &self.datetime_format {
            req = req.header("Accept-Datetime-Format", fmt.as_str());
        }
        req
    }

    pub fn print_json(&self, value: &serde_json::Value) {
        if self.pretty {
            println!("{}", serde_json::to_string_pretty(value).unwrap());
        } else {
            println!("{value}");
        }
    }

    async fn response_json(resp: reqwest::Response) -> OandaResult<serde_json::Value> {
        let status = resp.status();
        let body = Self::read_json_or_text(resp, status.is_success()).await?;

        if !status.is_success() {
            return Err(OandaError::Api { status, body });
        }

        Ok(body)
    }

    async fn read_json_or_text(
        resp: reqwest::Response,
        require_json: bool,
    ) -> OandaResult<serde_json::Value> {
        let text = resp.text().await.map_err(OandaError::Decode)?;
        match serde_json::from_str(&text) {
            Ok(body) => Ok(body),
            Err(err) if require_json => Err(OandaError::Json(err)),
            Err(_) => Ok(serde_json::Value::String(text)),
        }
    }

    pub async fn request_json(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<serde_json::Value>,
    ) -> OandaResult<serde_json::Value> {
        let mut req = self.request(method, path).query(query);
        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await.map_err(OandaError::Request)?;
        Self::response_json(resp).await
    }

    pub async fn get_json(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> OandaResult<serde_json::Value> {
        self.request_json(reqwest::Method::GET, path, query, None)
            .await
    }

    pub async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> OandaResult<serde_json::Value> {
        self.request_json(reqwest::Method::POST, path, &[], Some(body))
            .await
    }

    pub async fn put_json(
        &self,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> OandaResult<serde_json::Value> {
        self.request_json(reqwest::Method::PUT, path, &[], body)
            .await
    }

    pub async fn patch_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> OandaResult<serde_json::Value> {
        self.request_json(reqwest::Method::PATCH, path, &[], Some(body))
            .await
    }

    pub async fn stream_response(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> OandaResult<reqwest::Response> {
        let resp = self
            .stream_request(path)
            .query(query)
            .send()
            .await
            .map_err(OandaError::Request)?;

        let status = resp.status();
        if !status.is_success() {
            let body = Self::read_json_or_text(resp, false).await?;
            return Err(OandaError::Api { status, body });
        }

        Ok(resp)
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<(), String> {
        let body = Self::response_json(resp).await.map_err(|e| e.to_string())?;
        self.print_json(&body);
        Ok(())
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<(), String> {
        let resp = self
            .request(reqwest::Method::GET, path)
            .query(query)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<(), String> {
        let resp = self
            .request(reqwest::Method::POST, path)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn put(&self, path: &str, body: Option<serde_json::Value>) -> Result<(), String> {
        let mut req = self.request(reqwest::Method::PUT, path);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn patch(&self, path: &str, body: serde_json::Value) -> Result<(), String> {
        let resp = self
            .request(reqwest::Method::PATCH, path)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn stream(&self, path: &str, query: &[(&str, &str)]) -> Result<(), String> {
        let resp = self
            .stream_response(path, query)
            .await
            .map_err(|e| e.to_string())?;

        let mut stream = resp.bytes_stream();
        let mut buf = Vec::new();
        let mut stdout = std::io::stdout();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {e}"))?;
            buf.extend_from_slice(&chunk);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if self.pretty {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        let _ = writeln!(stdout, "{}", serde_json::to_string_pretty(&val).unwrap());
                    } else {
                        let _ = writeln!(stdout, "{line}");
                    }
                } else {
                    let _ = writeln!(stdout, "{line}");
                }
                let _ = stdout.flush();
            }
        }

        Ok(())
    }
}

pub fn read_body(body: Option<String>) -> Result<serde_json::Value, String> {
    match body {
        Some(b) => serde_json::from_str(&b).map_err(|e| format!("Invalid JSON body: {e}")),
        None => serde_json::from_reader(std::io::stdin())
            .map_err(|e| format!("Failed to read JSON from stdin: {e}")),
    }
}
