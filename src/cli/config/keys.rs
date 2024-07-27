use orch::lm::LanguageModelProvider;
use std::{cell::OnceCell, collections::HashMap, path::PathBuf};

use crate::{
    cli::subcommand_search::SearchConfig,
    core::{SuggestConfig, SuggestMode},
    lm::{AnthropicConfig, OllamaConfig, OpenAiConfig},
};

use super::{GeneralConfig, MagicCliConfig, MagicCliConfigError};

type ConfigurationKeyUpdateFn = Box<dyn Fn(&mut MagicCliConfig, &str) -> Result<(), MagicCliConfigError>>;

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
                "general.llm".to_string(),
                ConfigurationKey::new(
                    "general.llm".to_string(),
                    "The LLM to use for generating responses. Supported values: \"ollama\", \"openai\"".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.general.is_none() {
                            config.general = Some(GeneralConfig::default());
                        }
                        config.general.as_mut().unwrap().llm = Some(LanguageModelProvider::try_from(value).expect("Invalid LLM provider"));
                        Ok(())
                    })
                ).with_prio(0));
            keys.insert(
                "general.access_to_shell_history".to_string(),
                ConfigurationKey::new(
                    "general.access_to_shell_history".to_string(),
                    "Whether to allow access to the shell history.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.general.is_none() {
                            config.general = Some(GeneralConfig::default());
                        }
                        config.general.as_mut().unwrap().access_to_shell_history = Some(value.parse::<bool>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                        Ok(())
                    }),
                )
            );
            keys.insert(
                "suggest.mode".to_string(),
                ConfigurationKey::new(
                    "suggest.mode".to_string(),
                    "The mode to use for suggesting commands. Supported values: \"clipboard\" (copying command to clipboard), \"unsafe-execution\" (executing in the current shell session)".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
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
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.suggest.is_none() {
                            config.suggest = Some(SuggestConfig::default());
                        }
                        config.suggest.as_mut().unwrap().add_to_history = Some(value.parse::<bool>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                        Ok(())
                    }),
                )
            );
            keys.insert("search.allow_remote_llm".to_string(), ConfigurationKey::new(
                "search.allow_remote_llm".to_string(),
                "Whether to allow searching the command history using a remote LLM.".to_string(),
                Box::new(|config: &mut MagicCliConfig, value: &str| {
                    if config.search.is_none() {
                        config.search = Some(SearchConfig::default());
                    }
                    config.search.as_mut().unwrap().allow_remote_llm = Some(value.parse::<bool>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                    Ok(())
                }),
            ));
            keys.insert("search.shell_history".to_string(), ConfigurationKey::new(
                "search.shell_history".to_string(),
                "Optional path to the shell history file, otherwise uses the default according to the user's system.".to_string(),
                Box::new(|config: &mut MagicCliConfig, value: &str| {
                    if config.search.is_none() {
                        config.search = Some(SearchConfig::default());
                    }
                    config.search.as_mut().unwrap().shell_history = Some(value.parse::<PathBuf>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                    Ok(())
                }),
            ));
            keys.insert("search.index_dir".to_string(), ConfigurationKey::new(
                "search.index_dir".to_string(),
                "Optional path to the index directory, otherwise uses the default according to the user's system.".to_string(),
                Box::new(|config: &mut MagicCliConfig, value: &str| {
                    if config.search.is_none() {
                        config.search = Some(SearchConfig::default());
                    }
                    config.search.as_mut().unwrap().index_dir = Some(value.parse::<PathBuf>().map_err(|_| MagicCliConfigError::InvalidConfigValue(value.to_string()))?);
                    Ok(())
                }),
            ));
            keys.insert(
                "ollama.base_url".to_string(),
                ConfigurationKey::new(
                    "ollama.base_url".to_string(),
                    "The base URL of the Ollama API.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
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
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
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
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
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
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().api_key = Some(value.to_string());
                        Ok(())
                    }),
                ).secret()
            );
            keys.insert(
                "openai.api_endpoint".to_string(),
                ConfigurationKey::new(
                    "openai.api_endpoint".to_string(),
                    "Custom API endpoint for the OpenAI API.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().api_endpoint = Some(value.to_string());
                        Ok(())
                    }),
                ).secret()
            );
            keys.insert(
                "openai.model".to_string(),
                ConfigurationKey::new(
                    "openai.model".to_string(),
                    "The model to use for generating responses.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
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
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.openai_config.is_none() {
                            config.openai_config = Some(OpenAiConfig::default());
                        }
                        config.openai_config.as_mut().unwrap().embedding_model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            keys.insert(
                "anthropic.api_key".to_string(),
                ConfigurationKey::new(
                    "anthropic.api_key".to_string(),
                    "The API key for the Anthropic API.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.anthropic_config.is_none() {
                            config.anthropic_config = Some(AnthropicConfig::default());
                        }
                        config.anthropic_config.as_mut().unwrap().api_key = Some(value.to_string());
                        Ok(())
                    }),
                ).secret()
            );
            keys.insert(
                "anthropic.model".to_string(),
                ConfigurationKey::new(
                    "anthropic.model".to_string(),
                    "The model to use for generating responses.".to_string(),
                    Box::new(|config: &mut MagicCliConfig, value: &str| {
                        if config.anthropic_config.is_none() {
                            config.anthropic_config = Some(AnthropicConfig::default());
                        }
                        config.anthropic_config.as_mut().unwrap().model = Some(value.to_string());
                        Ok(())
                    }),
                )
            );
            keys
        });
        cell
    }
}
