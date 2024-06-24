use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use home::home_dir;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ollama::config::OllamaConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandSuggestMode {
    #[serde(rename = "clipboard")]
    Clipboard,
    #[serde(rename = "unsafe-execution")]
    Execution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(rename = "ollama")]
    pub ollama_config: OllamaConfig,
    pub suggest_mode: CommandSuggestMode,
}

impl Display for CliConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            ollama_config: Default::default(),
            suggest_mode: CommandSuggestMode::Clipboard,
        }
    }
}

#[derive(Error, Debug)]
pub enum CliConfigError {
    #[error("Configuration key not found: {0}")]
    MissingConfigKey(String),

    #[error("Could not parse configuration file: {0}")]
    ParsingError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Invalid config key: {0}")]
    InvalidConfigKey(String),

    #[error("Invalid config value: {0}")]
    InvalidConfigValue(String),
}

impl CliConfig {
    pub fn load_config() -> Result<CliConfig, CliConfigError> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            // Config doesn't exist - create it with default values.
            let config = CliConfig::default();
            let serialized_config = serde_json::to_string(&config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
            std::fs::write(config_path, serialized_config).map_err(|e| CliConfigError::IoError(e.to_string()))?;
            return Ok(config);
        }
        let deserialized_config =
            serde_json::from_str(&std::fs::read_to_string(config_path).map_err(|e| CliConfigError::IoError(e.to_string()))?)
                .map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        Ok(deserialized_config)
    }

    pub fn get_config_path() -> Result<PathBuf, CliConfigError> {
        let home = home_dir().unwrap();
        let config_path = home.join(".config").join("magic_cli.json");
        Ok(config_path)
    }

    pub fn reset() -> Result<(), CliConfigError> {
        let default_config = CliConfig::default();
        let serialized_config = serde_json::to_string(&default_config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(Self::get_config_path()?, serialized_config).map_err(|e| CliConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn get(key: &str) -> Result<String, CliConfigError> {
        // TODO: Make this generic (potentially support JSON path syntax).
        let config_path = Self::get_config_path()?;
        let config_content = std::fs::read_to_string(config_path).map_err(|e| CliConfigError::IoError(e.to_string()))?;
        let deserialized_config: serde_json::Value =
            serde_json::from_str(&config_content).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        let mut curr_value = deserialized_config.clone();
        for value in key.split('.') {
            curr_value = curr_value
                .get(value)
                .ok_or(CliConfigError::MissingConfigKey(key.to_string()))?
                .clone();
        }
        Ok(curr_value.to_string())
    }

    pub fn set(key: &str, value: &str) -> Result<(), CliConfigError> {
        let config_path = Self::get_config_path()?;
        let mut config = Self::load_config()?.clone();

        // TODO: This is manual and error-prone, should make this generic (potentially support JSON path syntax).
        match key {
            "suggest_mode" => {
                config.suggest_mode = match value {
                    "clipboard" => CommandSuggestMode::Clipboard,
                    "unsafe-execution" => CommandSuggestMode::Execution,
                    _ => return Err(CliConfigError::InvalidConfigValue(key.to_string())),
                }
            }
            "ollama.base_url" => {
                config.ollama_config.base_url = value.to_string();
            }
            "ollama.model" => {
                config.ollama_config.model = value.to_string();
            }
            _ => return Err(CliConfigError::InvalidConfigKey(key.to_string())),
        }

        let serialized_config = serde_json::to_string(&config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;

        std::fs::write(config_path, serialized_config).map_err(|e| CliConfigError::IoError(e.to_string()))?;
        Ok(())
    }
}
