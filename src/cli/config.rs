use thiserror::Error;

use crate::ollama::config::OllamaConfig;

#[derive(Debug)]
pub struct CliConfig {
    ollama_config: OllamaConfig,
}

#[derive(Error, Debug)]
pub enum CliConfigError {
    #[error("Configuration file not found: {0}")]
    MissingConfigFile(String),
}

impl CliConfig {
    pub fn load_config() -> Result<Self, CliConfigError> {
        // TODO: Load config from file.
        Ok(Self {
            ollama_config: Default::default(),
        })
    }
}
