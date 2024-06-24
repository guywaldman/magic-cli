use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use colored::Colorize;
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

impl Default for CommandSuggestMode {
    fn default() -> Self {
        Self::Clipboard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestConfig {
    pub mode: CommandSuggestMode,
    pub add_to_history: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(rename = "ollama")]
    pub ollama_config: OllamaConfig,
    pub suggest: SuggestConfig,
}

impl Display for CliConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
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
        let config_path = Self::get_config_file_path()?;
        if !config_path.exists() {
            // Config doesn't exist - create it with default values.
            println!(
                "{} '{}'.",
                "Configuration file not found, creating default configuration file at path".yellow(),
                config_path.to_str().unwrap().yellow()
            );
            let config = CliConfig::default();
            let serialized_config = serde_json::to_string(&config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
            std::fs::write(config_path, serialized_config).map_err(|e| CliConfigError::IoError(e.to_string()))?;
            println!("{}", "Default configuration file created successfully.".green());
            return Ok(config);
        }
        let deserialized_config =
            serde_json::from_str(&std::fs::read_to_string(config_path).map_err(|e| CliConfigError::IoError(e.to_string()))?)
                .map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        Ok(deserialized_config)
    }

    pub fn get_config_file_path() -> Result<PathBuf, CliConfigError> {
        let config_dir_path = Self::get_config_dir_path()?;
        let config_path = config_dir_path.join("config.json");
        Ok(config_path)
    }

    fn get_config_dir_path() -> Result<PathBuf, CliConfigError> {
        let home = home_dir().unwrap();
        let config_dir_path = home.join(".config").join("magic_cli");
        Ok(config_dir_path)
    }

    pub fn reset() -> Result<(), CliConfigError> {
        let default_config = CliConfig::default();
        let serialized_config = serde_json::to_string(&default_config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(Self::get_config_file_path()?, serialized_config).map_err(|e| CliConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    pub fn get(key: &str) -> Result<String, CliConfigError> {
        // TODO: Make this generic (potentially support JSON path syntax).
        let config_path = Self::get_config_file_path()?;
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
        let config_path = Self::get_config_file_path()?;
        let mut config = Self::load_config()?.clone();

        // TODO: This is manual and error-prone, should make this generic (potentially support JSON path syntax).
        match key {
            "suggest.mode" => {
                config.suggest.mode = match value {
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