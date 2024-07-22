use std::{cell::OnceCell, collections::HashMap};
use orch::lm::LanguageModelProvider;

use crate::{core::{SuggestConfig, SuggestMode}, lm::{OllamaConfig, OpenAiConfig}};

use super::{MagicCliConfigError, MagicCliConfigOptions};

type ConfigurationKeyUpdateFn = Box<dyn Fn(&mut MagicCliConfigOptions, &str) -> Result<(), MagicCliConfigError>>;

pub struct ConfigurationKey {
    pub key: String,
    pub description: String,
    pub is_secret: bool,
    pub prio: usize,
    pub update_fn: ConfigurationKeyUpdateFn,
}

impl ConfigurationKey {
    pub fn new(key: String, description: String, update_fn: ConfigurationKeyUpdateFn) -> Self {
        Self {
            key,
            description,
            is_secret: false,
            prio: 9999,
            update_fn,
        }
    }

    pub fn with_prio(mut self, prio: usize) -> Self {
        self.prio = prio;
        self
    }

    pub fn secret(mut self) -> Self {
        self.is_secret = true;
        self
    }
}

pub struct ConfigKeys;

impl ConfigKeys {
    pub fn keys() -> OnceCell<HashMap<String, ConfigurationKey>> {
        let cell = OnceCell::new();
        cell.get_or_init(|| {
            let mut keys = HashMap::new();
            keys.insert(
                "llm".to_string(),
                ConfigurationKey::new(
                    "llm".to_string(),
                    "The LLM to use for generating responses. Supported values: \"ollama\", \"openai\"".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        config.llm = Some(LanguageModelProvider::try_from(value).expect("Invalid LLM provider"));
                        Ok(())
                    })
                ).with_prio(0));
            keys.insert(
                "suggest.mode".to_string(),
                ConfigurationKey::new(
                    "suggest.mode".to_string(),
                    "The mode to use for suggesting commands. Supported values: \"clipboard\" (copying command to clipboard), \"unsafe-execution\" (executing in the current shell session)".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.suggest.is_none() {
                            config.suggest = Some(SuggestConfig::default());
                        }
                        config.suggest.as_mut().unwrap().mode = Some(SuggestMode::try_from(value)?);
                        Ok(())
                    }),
                )
            );
            keys.insert(
                "suggest.add_to_history".to_string(),
                ConfigurationKey::new(
                    "suggest.add_to_history".to_string(),
                    "Whether to add the suggested command to the shell history.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.suggest.is_none() {
                            config.suggest = Some(SuggestConfig::default());
                        }
                        config.suggest.as_mut().unwrap().add_to_history = Some(value.parse::<bool>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                        Ok(())
                    }),
                )
            );
            keys.insert(
                "ollama.base_url".to_string(),
                ConfigurationKey::new(
                    "ollama.base_url".to_string(),
                    "The base URL of the Ollama API.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.ollama_config.is_none() {
                            config.ollama_config = Some(OllamaConfig::default());
                        }
                        config.ollama_config.as_mut().unwrap().base_url = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            
            keys.insert(
                "ollama.model".to_string(),
                ConfigurationKey::new(
                    "ollama.model".to_string(),
                    "The model to use for generating responses.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.ollama_config.is_none() {
                            config.ollama_config = Some(OllamaConfig::default());
                        }
                        config.ollama_config.as_mut().unwrap().model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            
            keys.insert(
                "ollama.embedding_model".to_string(),
                ConfigurationKey::new(
                    "ollama.embedding_model".to_string(),
                    "The model to use for generating embeddings.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.ollama_config.is_none() {
                            config.ollama_config = Some(OllamaConfig::default());
                        }
                        config.ollama_config.as_mut().unwrap().embedding_model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            
            keys.insert(
                "openai.api_key".to_string(),
                ConfigurationKey::new(
                    "openai.api_key".to_string(),
                    "The API key for the OpenAI API.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().api_key = Some(value.to_string());
                        Ok(())
                    }),
                ).secret()
            );
            
            keys.insert(
                "openai.model".to_string(),
                ConfigurationKey::new(
                    "openai.model".to_string(),
                    "The model to use for generating responses.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            
            keys.insert(
                "openai.embedding_model".to_string(),
                ConfigurationKey::new(
                    "openai.embedding_model".to_string(),
                    "The model to use for generating embeddings.".to_string(),
                    Box::new(|config: &mut MagicCliConfigOptions, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().embedding_model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            keys
        });
        cell
    }
}
