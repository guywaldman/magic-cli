
use crate::llm::ollama::config::OllamaConfig;

use crate::llm::openai::config::OpenAiConfig;

use crate::cli::config::MagicCliConfigError;
use crate::core::{Llm, LlmProvider, SuggestConfig};
use colored::Colorize;
use home::home_dir;
use inquire::list_option::ListOption;
use inquire::Select;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use super::ConfigKeys;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MagicCliConfig {
    
    #[serde(rename = "ollama")]
    pub ollama_config: OllamaConfig,
    
    #[serde(rename = "openai")]
    pub openai_config: OpenAiConfig,
    pub llm: LlmProvider,
    pub suggest: SuggestConfig,
}

impl Display for MagicCliConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

impl MagicCliConfig {
    pub fn load_config() -> Result<MagicCliConfig, MagicCliConfigError> {
        let config_dir_path = Self::get_config_dir_path()?;
        if !config_dir_path.exists() {
            eprintln!(
                "{} '{}'.",
                "Configuration directory not found, creating default configuration directory at path".yellow(),
                config_dir_path.to_str().unwrap().yellow()
            );
            std::fs::create_dir_all(config_dir_path).map_err(MagicCliConfigError::IoError)?;
        }
        let config_path = Self::get_config_file_path()?;
        if !config_path.exists() {
            // Config doesn't exist - create it with default values.
            eprintln!(
                "{} '{}'.",
                "Configuration file not found, creating default configuration file at path".yellow(),
                config_path.to_str().unwrap().yellow()
            );
            let config = MagicCliConfig::default();
            let serialized_config = serde_json::to_string(&config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
            std::fs::write(config_path, serialized_config).map_err(MagicCliConfigError::IoError)?;
            println!("{}", "Default configuration file created successfully.".green());
            return Ok(config);
        }
        let deserialized_config = serde_json::from_str(&std::fs::read_to_string(config_path).map_err(MagicCliConfigError::IoError)?)
            .map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        Ok(deserialized_config)
    }

    pub fn get_config_file_path() -> Result<PathBuf, MagicCliConfigError> {
        let config_dir_path = Self::get_config_dir_path()?;
        let config_path = config_dir_path.join("config.json");
        Ok(config_path)
    }

    pub fn get_config_dir_path() -> Result<PathBuf, MagicCliConfigError> {
        let home = home_dir().unwrap();
        let config_dir_path = home.join(".config").join("magic_cli");
        Ok(config_dir_path)
    }

    pub fn reset() -> Result<(), MagicCliConfigError> {
        let default_config = MagicCliConfig::default();
        let serialized_config = serde_json::to_string(&default_config).map_err(|e| MagicCliConfigError::ParsingError(e.to_string()))?;
        std::fs::write(Self::get_config_file_path()?, serialized_config).map_err(MagicCliConfigError::IoError)?;
        Ok(())
    }

    // TODO: Support arrays.
    pub fn get(key: &str) -> Result<String, MagicCliConfigError> {
        let config_path = Self::get_config_file_path()?;
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

    pub fn set(key: &str, value: &str) -> Result<(), MagicCliConfigError> {
        let config_path = Self::get_config_file_path()?;
        let mut config = Self::load_config()?.clone();
        let config_keys = ConfigKeys::keys();
        let config_keys = config_keys.get().unwrap();
        if !config_keys.contains_key(key) {
            return Err(MagicCliConfigError::InvalidConfigKey(key.to_string()));
        }
        let config_value = config_keys.get(key).unwrap();
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

    pub fn llm_from_config(config: &MagicCliConfig) -> Result<Box<dyn Llm>, Box<dyn Error>> {
        
        if config.llm == LlmProvider::Ollama {
            use crate::llm::ollama::ollama_llm::OllamaLocalLlm;
            return Ok(Box::new(OllamaLocalLlm::new(config.ollama_config.clone())));
        }
        
        if config.llm == LlmProvider::OpenAi {
            use crate::llm::openai::openai_llm::OpenAiLlm;
            return Ok(Box::new(OpenAiLlm::new(config.openai_config.clone())));
        }
        Err(Box::new(MagicCliConfigError::LlmNotSupported {
            llm: config.llm.to_string(),
            feature: "ollama".to_string(),
        }))
    }
}
