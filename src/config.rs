use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub seed: SeedConfig,
    pub server: ServerConfig,
    #[serde(rename = "loop")]
    pub loop_config: LoopConfig,
    pub logging: LoggingConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SeedConfig {
    pub endpoint: String,
    pub token_path: PathBuf,
    #[serde(default)]
    pub allow_invalid_certs: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub bind: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoopConfig {
    pub poll_interval_ms: u64,
    pub http_timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&raw)?;
        if config.loop_config.poll_interval_ms == 0 {
            return Err(Error::Config("poll_interval_ms must be > 0".to_string()));
        }
        Ok(config)
    }

    pub fn http_timeout(&self) -> Duration {
        Duration::from_millis(self.loop_config.http_timeout_ms)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.loop_config.poll_interval_ms)
    }

    pub fn load_token(&self) -> Result<String> {
        if let Ok(token) = std::env::var("MICROCCF_SEED_TOKEN") {
            if !token.trim().is_empty() {
                return Ok(token);
            }
        }
        let token = fs::read_to_string(&self.seed.token_path)?;
        let token = token.trim().to_string();
        if token.is_empty() {
            return Err(Error::Config("token file is empty".to_string()));
        }
        Ok(token)
    }
}
