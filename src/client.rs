use futures_util::StreamExt;
use reqwest::Client;
use std::io::Write;

use crate::config::Config;

pub struct OandaClient {
    http: Client,
    base_url: String,
    stream_url: String,
    token: String,
    datetime_format: Option<String>,
    pretty: bool,
}

impl OandaClient {
    pub fn new(config: &Config) -> Result<Self, String> {
        let token = config.require_token()?.to_string();
        Ok(OandaClient {
            http: Client::new(),
            base_url: config.environment.base_url().to_string(),
            stream_url: config.environment.stream_url().to_string(),
            token,
            datetime_format: config.datetime_format.clone(),
            pretty: config.pretty,
        })
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        let mut req = self.http.request(method, &url)
            .bearer_auth(&self.token);
        if let Some(fmt) = &self.datetime_format {
            req = req.header("Accept-Datetime-Format", fmt.as_str());
        }
        req
    }

    fn stream_request(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{path}", self.stream_url);
        let mut req = self.http.get(&url)
            .bearer_auth(&self.token);
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

    async fn handle_response(&self, resp: reqwest::Response) -> Result<(), String> {
        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to read response: {e}"))?;

        if !status.is_success() {
            eprintln!("{}", serde_json::to_string_pretty(&body).unwrap());
            std::process::exit(1);
        }

        self.print_json(&body);
        Ok(())
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<(), String> {
        let resp = self.request(reqwest::Method::GET, path)
            .query(query)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<(), String> {
        let resp = self.request(reqwest::Method::POST, path)
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
        let resp = req.send().await.map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn patch(&self, path: &str, body: serde_json::Value) -> Result<(), String> {
        let resp = self.request(reqwest::Method::PATCH, path)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;
        self.handle_response(resp).await
    }

    pub async fn stream(&self, path: &str, query: &[(&str, &str)]) -> Result<(), String> {
        let resp = self.stream_request(path)
            .query(query)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to read response: {e}"))?;
            eprintln!("{}", serde_json::to_string_pretty(&body).unwrap());
            std::process::exit(1);
        }

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
        None => serde_json::from_reader(std::io::stdin()).map_err(|e| format!("Failed to read JSON from stdin: {e}")),
    }
}
