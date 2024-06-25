use crate::core::{SuggestConfig, SuggestMode, SuggestModeError};
use crate::llm::ollama::config::OllamaConfig;
use crate::llm::openai::config::OpenAiConfig;
use colored::Colorize;
use home::home_dir;
use inquire::list_option::ListOption;
use inquire::{InquireError, Select};
use serde::{Deserialize, Serialize};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};
use thiserror::Error;

type ConfigurationKeyUpdateFn = Box<dyn Fn(&mut CliConfig, &str) -> Result<(), CliConfigError>>;

pub struct ConfigurationKey {
    pub key: String,
    pub description: String,
    update_fn: ConfigurationKeyUpdateFn,
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
    type Error = CliConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ollama" => Ok(LlmProvider::Ollama),
            "openai" => Ok(LlmProvider::OpenAi),
            _ => Err(CliConfigError::InvalidConfigValue(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(rename = "ollama")]
    pub ollama_config: OllamaConfig,
    #[serde(rename = "openai")]
    pub openai_config: OpenAiConfig,
    pub llm: LlmProvider,
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

impl CliConfig {
    pub fn load_config() -> Result<CliConfig, CliConfigError> {
        let config_dir_path = Self::get_config_dir_path()?;
        if !config_dir_path.exists() {
            eprintln!(
                "{} '{}'.",
                "Configuration directory not found, creating default configuration directory at path".yellow(),
                config_dir_path.to_str().unwrap().yellow()
            );
            std::fs::create_dir_all(config_dir_path).map_err(CliConfigError::IoError)?;
        }
        let config_path = Self::get_config_file_path()?;
        if !config_path.exists() {
            // Config doesn't exist - create it with default values.
            eprintln!(
                "{} '{}'.",
                "Configuration file not found, creating default configuration file at path".yellow(),
                config_path.to_str().unwrap().yellow()
            );
            let config = CliConfig::default();
            let serialized_config = serde_json::to_string(&config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
            std::fs::write(config_path, serialized_config).map_err(CliConfigError::IoError)?;
            println!("{}", "Default configuration file created successfully.".green());
            return Ok(config);
        }
        let deserialized_config = serde_json::from_str(&std::fs::read_to_string(config_path).map_err(CliConfigError::IoError)?)
            .map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        Ok(deserialized_config)
    }

    pub fn get_config_file_path() -> Result<PathBuf, CliConfigError> {
        let config_dir_path = Self::get_config_dir_path()?;
        let config_path = config_dir_path.join("config.json");
        Ok(config_path)
    }

    pub fn get_config_dir_path() -> Result<PathBuf, CliConfigError> {
        let home = home_dir().unwrap();
        let config_dir_path = home.join(".config").join("magic_cli");
        Ok(config_dir_path)
    }

    pub fn reset() -> Result<(), CliConfigError> {
        let default_config = CliConfig::default();
        let serialized_config = serde_json::to_string(&default_config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(Self::get_config_file_path()?, serialized_config).map_err(CliConfigError::IoError)?;
        Ok(())
    }

    // TODO: Support arrays.
    pub fn get(key: &str) -> Result<String, CliConfigError> {
        let config_path = Self::get_config_file_path()?;
        let config_content = std::fs::read_to_string(config_path).map_err(CliConfigError::IoError)?;
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
        let config_keys = Self::configuration_keys();
        let config_keys = config_keys.get().unwrap();
        if !config_keys.contains_key(key) {
            return Err(CliConfigError::InvalidConfigKey(key.to_string()));
        }
        let config_value = config_keys.get(key).unwrap();
        (config_value.update_fn)(&mut config, value)?;

        let serialized_config = serde_json::to_string(&config).map_err(|e| CliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(config_path, serialized_config).map_err(CliConfigError::IoError)?;
        Ok(())
    }

    pub fn select_key() -> Result<String, CliConfigError> {
        let config = CliConfig::configuration_keys();
        let config = config.get().unwrap();
        // Represents (index, key, description) tuples.
        let items = config
            .values()
            .enumerate()
            .map(|(i, item)| (i, item.key.clone(), format!("{}: {}", item.key, item.description.dimmed())))
            .collect::<Vec<_>>();

        let choice = Select::new(
            "Select the configuration key:",
            items.iter().map(|it| ListOption::new(it.0, it.2.clone())).collect(),
        )
        .prompt()?;
        let choice_str = choice.to_string();
        let choice = items.iter().find(|it| it.2 == choice_str).unwrap();
        Ok(choice.1.to_string())
    }

    pub fn configuration_keys() -> OnceCell<HashMap<String, ConfigurationKey>> {
        let cell = OnceCell::new();
        cell.get_or_init(|| {
            let mut keys = HashMap::new();
            keys.insert(
                "llm".to_string(),
                ConfigurationKey {
                    key: "llm".to_string(),
                    description: "The LLM to use for generating responses.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.llm = LlmProvider::try_from(value)?;
                        Ok(())
                    }),
                },
            );
            keys.insert("suggest.mode".to_string(), ConfigurationKey {
                key: "suggest.mode".to_string(),
                description: "The mode to use for suggesting commands (supported values: \"clipboard\" for copying to clipboard, \"unsafe-execution\" for executing commands in the current shell session).".to_string(),
                update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                    config.suggest.mode = SuggestMode::try_from(value)?;
                    Ok(())
                }),
            });
            keys.insert("suggest.add_to_history".to_string(), ConfigurationKey {
                key: "suggest.add_to_history".to_string(),
                description: "Whether to add the suggested command to the shell history (supported values: \"true\", \"false\").".to_string(),
                update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                    config.suggest.add_to_history = value.parse::<bool>().map_err(|_| CliConfigError::InvalidConfigValue(value.to_string()))?;
                    Ok(())
                }),
            });
            keys.insert(
                "ollama.base_url".to_string(),
                ConfigurationKey {
                    key: "ollama.base_url".to_string(),
                    description: "The base URL of the Ollama API.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.ollama_config.base_url = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys.insert(
                "ollama.model".to_string(),
                ConfigurationKey {
                    key: "ollama.model".to_string(),
                    description: "The model to use for generating responses.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.ollama_config.model = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys.insert(
                "ollama.embedding_model".to_string(),
                ConfigurationKey {
                    key: "ollama.embedding_model".to_string(),
                    description: "The model to use for generating embeddings.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.ollama_config.embedding_model = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys.insert(
                "openai.api_key".to_string(),
                ConfigurationKey {
                    key: "openai.api_key".to_string(),
                    description: "The API key for the OpenAI API.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.openai_config.api_key = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys.insert(
                "openai.model".to_string(),
                ConfigurationKey {
                    key: "openai.model".to_string(),
                    description: "The model to use for generating responses.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.openai_config.model = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys.insert(
                "openai.embedding_model".to_string(),
                ConfigurationKey {
                    key: "openai.embedding_model".to_string(),
                    description: "The model to use for generating embeddings.".to_string(),
                    update_fn: Box::new(|config: &mut CliConfig, value: &str| {
                        config.openai_config.embedding_model = Some(value.to_string());
                        Ok(())
                    }),
                },
            );
            keys
        });

        cell
    }
}
