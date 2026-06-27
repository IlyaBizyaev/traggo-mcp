#![warn(clippy::pedantic)]
mod config;
mod cursor;
mod error;
mod graphql;
mod tools;
mod traggo_client;
mod validation;

use anyhow::Context;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

use crate::{config::Config, tools::TraggoTools, traggo_client::TraggoClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let config = Config::from_env().context("failed to load Traggo MCP configuration")?;
    let client = TraggoClient::new(config).context("failed to initialize Traggo client")?;
    let service = TraggoTools::new(client)
        .serve(stdio())
        .await
        .inspect_err(|error| tracing::error!(?error, "stdio serving error"))?;
    service.waiting().await?;
    Ok(())
}
