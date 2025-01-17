use inquire::InquireError;
use thiserror::Error;

use crate::core::SuggestModeError;

#[derive(Error, Debug)]
pub enum MagicCliConfigError {
    #[error("Configuration key not found: {0}")]
    MissingConfigKey(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Could not parse configuration file: {0}")]
    ParsingError(String),

    #[error("I/O error: {0}: {0}")]
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

pub(crate) trait ConfigOptions {
    /// Populates the default values for the configuration options.
    /// Returns `true` if the configuration was populated, `false` if it was already populated.
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError>;
}
