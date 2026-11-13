use std::time::Duration;

use async_trait::async_trait;
use clap::ValueEnum;
use reqwest::Client;
use sigstore_rekor::client::{RekorClient, RekorV2Client};

use super::error::{ClientRekorError, RekorApiError};
use super::result::LeakResult;

pub const DEFAULT_SERVER: &str = "https://log2025-1.rekor.sigstore.dev";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, ValueEnum)]
pub enum RekorVersion {
    V1,
    #[default]
    V2,
}

#[async_trait]
pub trait RekorApi: Send + Sync {
    async fn get_entry_by_index(&self, index: u64) -> Result<LeakResult, RekorApiError>;
    async fn get_tile_containing(&self, index: u64) -> Result<Vec<LeakResult>, RekorApiError> {
        Ok(vec![self.get_entry_by_index(index).await?])
    }
    fn scan_unit_width(&self) -> u64 {
        1
    }
}

/// Client configuration; `Default` targets the Sigstore public-good log.
#[derive(Debug, Clone)]
pub struct Config {
    server: String,
    timeout: Duration,
    pool_max_idle_per_host: usize,
    version: RekorVersion,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: DEFAULT_SERVER.to_owned(),
            timeout: Duration::from_secs(10),
            pool_max_idle_per_host: 10,
            version: RekorVersion::default(),
        }
    }
}

impl Config {
    pub fn with_server(mut self, server: impl Into<String>) -> Self {
        self.server = server.into().trim_end_matches('/').to_owned();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_version(mut self, version: RekorVersion) -> Self {
        self.version = version;
        self
    }

    /// Build the concrete client behind the version-agnostic [`RekorApi`] trait.
    pub fn build_api(&self) -> Result<Box<dyn RekorApi>, ClientRekorError> {
        let http = Client::builder()
            .timeout(self.timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .user_agent(concat!("rekor-leak/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(ClientRekorError::from)?;

        Ok(match self.version {
            RekorVersion::V1 => Box::new(
                RekorClient::builder(&self.server)
                    .with_http_client(http)
                    .build()?,
            ),
            RekorVersion::V2 => Box::new(RekorV2Client::with_http_client(&self.server, http)),
        })
    }
}
