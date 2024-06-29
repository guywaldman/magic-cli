use inquire::InquireError;
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

    #[error("LLM {llm} not supported, please make sure you compiled with the '{feature}' feature enabled")]
    LlmNotSupported { llm: String, feature: String },
}
