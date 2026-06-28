//! Library interface for `oanda-cli`.
//!
//! The crate exposes the same thin OANDA v20 REST wrapper used by the CLI.
//! Library callers can use resource helpers such as [`OandaClient::account`]
//! and [`OandaClient::pricing`], or drop down to the raw `*_json` methods.
//!
//! ```no_run
//! use oanda_cli::{Config, OandaClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::new("token", "101-001-12345678-001");
//! let client = OandaClient::new(&config)?;
//! let account = client.account().summary().await?;
//! println!("{account}");
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod client;
pub mod commands;
pub mod config;

pub mod labs {
    //! Unofficial OANDA Labs API helpers.

    pub use crate::commands::labs::{BookType, Instrument, fetch_book};
}

pub use api::*;
pub use client::{OandaClient, OandaError, OandaResult, read_body};
pub use config::{Config, Environment};
