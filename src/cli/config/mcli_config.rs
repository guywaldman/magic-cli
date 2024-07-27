use crate::cli::subcommand_search::SearchConfig;
use crate::core::SuggestConfig;
use crate::lm::{AnthropicConfig, OllamaConfig, OpenAiConfig};

use crate::cli::config::{ConfigOptions, MagicCliConfigError};
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

use super::{ConfigKeys, GeneralConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicCliConfig {
    /// General configuration options.
    pub general: Option<GeneralConfig>,

    /// Configuration for command suggestions (e.g., the `suggest` subcommand).
    pub suggest: Option<SuggestConfig>,

    /// Configuration for the `search` subcommand.
    pub search: Option<SearchConfig>,

    /// Options for the Ollama LLM provider.
    #[serde(rename = "ollama")]
    pub ollama_config: Option<OllamaConfig>,

    /// Options for the OpenAI LLM provider.
    #[serde(rename = "openai")]
    pub openai_config: Option<OpenAiConfig>,

    /// Options for the Anthropic LLM provider.
    #[serde(rename = "anthropic")]
    pub anthropic_config: Option<AnthropicConfig>,
}

impl ConfigOptions for MagicCliConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;

        match self.general.as_mut() {
            Some(general_config) => {
                populated |= general_config.populate_defaults()?;
            }
            None => {
                populated = true;
                self.general = Some(GeneralConfig::default());
            }
        }
        match self.suggest.as_mut() {
            Some(suggest_config) => {
                populated |= suggest_config.populate_defaults()?;
            }
            None => {
                populated = true;
                self.suggest = Some(SuggestConfig::default());
            }
        }
        match self.search.as_mut() {
            Some(search_config) => {
                populated |= search_config.populate_defaults()?;
            }
            None => {
                populated = true;
                self.search = Some(SearchConfig::default());
            }
        }
        match self.ollama_config.as_mut() {
            Some(ollama_config) => {
                populated |= ollama_config.populate_defaults()?;
            }
            None => {
                populated = true;
                self.ollama_config = Some(OllamaConfig::default());
            }
        }
        match self.openai_config.as_mut() {
            Some(openai_config) => {
                populated |= openai_config.populate_defaults()?;
            }
            None => {
                populated = true;
                self.openai_config = Some(OpenAiConfig::default());
            }
        }

        Ok(populated)
    }
}

impl Default for MagicCliConfig {
    fn default() -> Self {
        Self {
            general: Some(GeneralConfig::default()),
            ollama_config: Some(OllamaConfig::default()),
            openai_config: Some(OpenAiConfig::default()),
            anthropic_config: Some(AnthropicConfig::default()),
            suggest: Some(SuggestConfig::default()),
            search: Some(SearchConfig::default()),
        }
    }
}

impl Display for MagicCliConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct MagicCliConfigManager {
    pub path: PathBuf,
    pub config: MagicCliConfig,
}

impl MagicCliConfigManager {
    pub fn try_from_path(config_path: &PathBuf) -> Result<Self, MagicCliConfigError> {
        let config = MagicCliConfigManager::load_config_from_path(config_path).map_err(|e| {
            MagicCliConfigError::Configuration(format!("Error when loading config file '{}': {}", config_path.display(), e))
        })?;
        Ok(Self {
            config,
            path: config_path.clone(),
        })
    }

    pub fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut new_config = self.config.clone();
        let populated = new_config.populate_defaults()?;

        if populated {
            self.save_config(&self.path, &new_config)?;
            self.config = new_config;
        }

        Ok(populated)
    }

    pub fn save_config(&self, config_path: &PathBuf, config: &MagicCliConfig) -> Result<(), MagicCliConfigError> {
        let serialized_config = serde_json::to_string_pretty(&config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(config_path, serialized_config).map_err(MagicCliConfigError::IoError)?;
        Ok(())
    }

    // TODO: Support arrays.
    pub fn get(&self, key: &str) -> Result<String, MagicCliConfigError> {
        let config_content = std::fs::read_to_string(&self.path).map_err(MagicCliConfigError::IoError)?;
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
        let config_keys = ConfigKeys::keys();
        let config_keys = config_keys.get().unwrap();
        if !config_keys.contains_key(key) {
            return Err(MagicCliConfigError::InvalidConfigKey(key.to_string()));
        }
        let config_value = config_keys.get(key).unwrap();

        // Change the value in the config.
        let mut config = Self::load_config_from_path(&self.path)?;
        (config_value.update_fn)(&mut config, value)?;

        self.save_config(&self.path, &config)?;
        Ok(())
    }

    pub fn prompt_user_to_select_key() -> Result<String, MagicCliConfigError> {
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
        let config = Self::load_config_from_path(&self.path)?;
        let Some(general_config) = config.general else {
            return Err(MagicCliConfigError::MissingConfigKey("general".to_owned()));
        };
        let Some(llm) = general_config.llm else {
            return Err(MagicCliConfigError::MissingConfigKey("general.llm".to_owned()));
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
                let Some(api_key) = openai_config.api_key.clone() else {
                    return Err(MagicCliConfigError::MissingConfigKey("api_key".to_owned()));
                };
                let mut openai_builder = OpenAiBuilder::new()
                    .with_model(model)
                    .with_embeddings_model(embedding_model)
                    .with_api_key(api_key);
                if let Some(api_endpoint) = openai_config.api_endpoint.clone() {
                    openai_builder = openai_builder.with_api_endpoint(api_endpoint);
                }
                let openai = openai_builder
                    .try_build()
                    .map_err(|e| MagicCliConfigError::Configuration(e.to_string()))?;
                Ok(Box::new(openai))
            }
            LanguageModelProvider::Anthropic => {
                let Some(anthropic_config) = config.anthropic_config else {
                    return Err(MagicCliConfigError::MissingConfigKey("anthropic".to_owned()));
                };
            }
        }
    }

    pub fn load_config_from_path(config_path: &PathBuf) -> Result<MagicCliConfig, MagicCliConfigError> {
        let mut config: MagicCliConfig = serde_json::from_str(&std::fs::read_to_string(config_path).map_err(MagicCliConfigError::IoError)?)
            .map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;

        // Populate defaults for missing keys.
        config.populate_defaults()?;
        Ok(config)
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

    pub fn reset(&mut self) -> Result<(), MagicCliConfigError> {
        let defaults = MagicCliConfig::default();
        self.config = defaults;
        self.save_config(&self.path, &self.config)?;
        Ok(())
    }
}
