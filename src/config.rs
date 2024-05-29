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
    pub trusted_sources: Vec<IpAddr>,
}

impl FirewallConfig {
    pub fn is_enabled(&self) -> bool {
        !self.trusted_sources.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CdnConfig {
    pub storage_path: Option<String>,
    pub firewall: FirewallConfig,
}

pub fn get_config() -> Result<CdnConfig> {
    let config_path = config_location().join("config.toml");
    Ok(confy::load_path::<CdnConfig>(config_path)?)
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
