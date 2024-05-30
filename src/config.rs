use std::{
    env,
    net::IpAddr,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FirewallConfig {
    pub enabled: bool,
    pub trusted_sources: Vec<IpAddr>,
}

impl FirewallConfig {
    fn validate(&self) {
        if self.enabled && self.trusted_sources.is_empty() {
            error!("Trusted sources cannot be empty if firewall is enabled.");
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CdnConfig {
    pub storage_path: Option<String>,
    pub firewall: FirewallConfig,
}

pub fn get_config() -> Result<CdnConfig> {
    let config_path = config_location().join("config.toml");
    let config: CdnConfig = confy::load_path(config_path)?;
    config.firewall.validate();
    Ok(config)
}

pub fn config_location() -> PathBuf {
    if cfg!(debug_assertions) {
        env::current_dir()
            .unwrap_or_else(|_| error!("Current directory inaccessible"))
            .join("assets/")
    } else {
        Path::new("/etc/rs_cdn/").to_path_buf()
    }
}
