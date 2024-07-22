use crate::lm::{OllamaConfig, OpenAiConfig};

use crate::cli::config::MagicCliConfigError;
use crate::core::SuggestConfig;
use colored::Colorize;
use home::home_dir;
use inquire::list_option::ListOption;
use inquire::Select;
use orch::lm::{LanguageModel, LanguageModelBuilder, LanguageModelProvider, OllamaBuilder, OpenAiBuilder};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use super::ConfigKeys;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MagicCliConfigOptions {
    #[serde(rename = "ollama")]
    pub ollama_config: Option<OllamaConfig>,

    #[serde(rename = "openai")]
    pub openai_config: Option<OpenAiConfig>,
    pub llm: Option<LanguageModelProvider>,
    pub suggest: Option<SuggestConfig>,
}

impl Display for MagicCliConfigOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct MagicCliConfig {
    /// The path to the configuration file.
    /// The default is used (`~/.config/magic_cli/config.json`) if this is `None`.
    pub config_path: Option<PathBuf>,
}

impl MagicCliConfig {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }

    pub fn load_config(&self) -> Result<MagicCliConfigOptions, MagicCliConfigError> {
        if let Some(config_path) = &self.config_path {
            // Custom config is used.
            return Self::load_config_from_path(config_path);
        }

        // Default config is used.

        let config_dir_path = Self::get_config_default_dir_path()?;
        if !config_dir_path.exists() {
            eprintln!(
                "{} '{}'.",
                "Configuration directory not found, creating default configuration directory at path".yellow(),
                config_dir_path.to_str().unwrap().yellow()
            );
            std::fs::create_dir_all(config_dir_path).map_err(MagicCliConfigError::IoError)?;
        }
        let config_path = Self::get_default_config_file_path()?;
        if !config_path.exists() {
            // Config doesn't exist - create it with default values.
            eprintln!(
                "{} '{}'.",
                "Configuration file not found, creating default configuration file at path".yellow(),
                config_path.to_str().unwrap().yellow()
            );
            let config = MagicCliConfigOptions::default();
            let serialized_config = serde_json::to_string(&config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
            std::fs::write(config_path, serialized_config).map_err(MagicCliConfigError::IoError)?;
            println!("{}", "Default configuration file created successfully.".green());
            return Ok(config);
        }

        Self::load_config_from_path(&config_path)
    }

    // TODO: Support arrays.
    pub fn get(&self, key: &str) -> Result<String, MagicCliConfigError> {
        let config_path = self.get_config_file_path()?;
        let config_content = std::fs::read_to_string(config_path).map_err(MagicCliConfigError::IoError)?;
        let deserialized_config: serde_json::Value =
            serde_json::from_str(&config_content).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        let mut curr_value = deserialized_config.clone();
        for value in key.split('.') {
            curr_value = curr_value
                .get(value)
                .ok_or(MagicCliConfigError::MissingConfigKey(key.to_string()))?
                .clone();
        }
        Ok(curr_value.to_string())
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), MagicCliConfigError> {
        let config_path = self.get_config_file_path()?;
        let config_keys = ConfigKeys::keys();
        let config_keys = config_keys.get().unwrap();
        if !config_keys.contains_key(key) {
            return Err(MagicCliConfigError::InvalidConfigKey(key.to_string()));
        }
        let config_value = config_keys.get(key).unwrap();

        // Change the value in the config.
        let mut config = Self::load_config_from_path(&config_path)?;
        (config_value.update_fn)(&mut config, value)?;

        let serialized_config = serde_json::to_string(&config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(config_path, serialized_config).map_err(MagicCliConfigError::IoError)?;
        Ok(())
    }

    pub fn select_key() -> Result<String, MagicCliConfigError> {
        let config = ConfigKeys::keys();
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

    pub fn lm_from_config(&self) -> Result<Box<dyn LanguageModel>, MagicCliConfigError> {
        let config_path = self.get_config_file_path()?;
        let config = Self::load_config_from_path(&config_path)?;
        let Some(llm) = config.llm else {
            return Err(MagicCliConfigError::MissingConfigKey("llm".to_owned()));
        };
        match llm {
            LanguageModelProvider::Ollama => {
                let Some(ollama_config) = config.ollama_config else {
                    return Err(MagicCliConfigError::MissingConfigKey("ollama".to_owned()));
                };
                let Some(model) = ollama_config.model.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("model".to_owned()));
                };
                let Some(embedding_model) = ollama_config.embedding_model.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("embedding_model".to_owned()));
                };
                let Some(base_url) = ollama_config.base_url.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("base_url".to_owned()));
                };
                let ollama = OllamaBuilder::new()
                    .with_base_url(base_url)
                    .with_model(model)
                    .with_embeddings_model(embedding_model)
                    .try_build()
                    .map_err(|e| MagicCliConfigError::Configuration(e.to_string()))?;
                Ok(Box::new(ollama))
            }
            LanguageModelProvider::OpenAi => {
                let Some(openai_config) = config.openai_config else {
                    return Err(MagicCliConfigError::MissingConfigKey("openai".to_owned()));
                };
                let Some(model) = openai_config.model.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("model".to_owned()));
                };
                let Some(embedding_model) = openai_config.embedding_model.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("embedding_model".to_owned()));
                };
                let openai = OpenAiBuilder::new()
                    .with_model(model)
                    .with_embeddings_model(embedding_model)
                    .try_build()
                    .map_err(|e| MagicCliConfigError::Configuration(e.to_string()))?;
                Ok(Box::new(openai))
            }
        }
    }

    pub fn load_config_from_path(config_path: &PathBuf) -> Result<MagicCliConfigOptions, MagicCliConfigError> {
        let deserialized_config = serde_json::from_str(&std::fs::read_to_string(config_path).map_err(MagicCliConfigError::IoError)?)
            .map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        Ok(deserialized_config)
    }

    pub fn get_config_file_path(&self) -> Result<PathBuf, MagicCliConfigError> {
        if let Some(config_path) = &self.config_path {
            return Ok(config_path.clone());
        }

        Self::get_default_config_file_path()
    }

    pub fn get_default_config_file_path() -> Result<PathBuf, MagicCliConfigError> {
        let config_dir_path = Self::get_config_default_dir_path()?;
        let config_path = config_dir_path.join("config.json");
        Ok(config_path)
    }

    pub fn get_config_default_dir_path() -> Result<PathBuf, MagicCliConfigError> {
        let home = home_dir().unwrap();
        let config_dir_path = home.join(".config").join("magic_cli");
        Ok(config_dir_path)
    }

    pub fn reset() -> Result<(), MagicCliConfigError> {
        let default_config = MagicCliConfigOptions::default();
        let serialized_config = serde_json::to_string(&default_config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(Self::get_default_config_file_path()?, serialized_config).map_err(MagicCliConfigError::IoError)?;
        Ok(())
    }
}
