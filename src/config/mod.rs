use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PLuaConfig {
    pub enabled_plugins: Vec<String>,
}

impl Default for PLuaConfig {
    fn default() -> Self {
        Self {
            enabled_plugins: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct ConfigManager {
    config_path: PathBuf,
    pub config: PLuaConfig,
}

impl ConfigManager {
    pub fn new(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir).context("Failed to create data directory")?;

        let config_path = data_dir.join("config.json");
        let config = if config_path.exists() {
            let config_str =
                fs::read_to_string(&config_path).context("Failed to read config file")?;
            serde_json::from_str(&config_str).context("Failed to parse config file")?
        } else {
            let default_config = PLuaConfig::default();
            let config_str = serde_json::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;
            fs::write(&config_path, config_str).context("Failed to write default config file")?;
            default_config
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn save(&self) -> Result<()> {
        let config_str =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;
        fs::write(&self.config_path, config_str).context("Failed to write config file")?;
        Ok(())
    }

    pub fn enable_plugin(&mut self, plugin_name: String) -> Result<bool> {
        if !self.config.enabled_plugins.contains(&plugin_name) {
            self.config.enabled_plugins.push(plugin_name);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn disable_plugin(&mut self, plugin_name: &str) -> Result<bool> {
        let initial_len = self.config.enabled_plugins.len();
        self.config
            .enabled_plugins
            .retain(|name| name != plugin_name);

        if self.config.enabled_plugins.len() < initial_len {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
