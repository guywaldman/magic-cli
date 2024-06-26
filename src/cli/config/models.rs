use inquire::InquireError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::SuggestModeError;

#[derive(Error, Debug)]
pub enum MagicCliConfigError {
    #[error("Configuration key not found: {0}")]
    MissingConfigKey(String),

    #[error("Could not parse configuration file: {0}")]
    ParsingError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid config key: {0}")]
    InvalidConfigKey(String),

    #[error("Invalid config value: {0}")]
    InvalidConfigValue(String),

    #[error("Selection error: {0}")]
    SelectError(#[from] InquireError),

    #[error("Error converting from or to 'SuggestMode': {0}")]
    SuggestModeError(#[from] SuggestModeError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAi,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Ollama
    }
}

impl TryFrom<&str> for LlmProvider {
    type Error = MagicCliConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ollama" => Ok(LlmProvider::Ollama),
            "openai" => Ok(LlmProvider::OpenAi),
            _ => Err(MagicCliConfigError::InvalidConfigValue(value.to_string())),
        }
    }
}
