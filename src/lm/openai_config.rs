use orch::lm::{openai_embedding_model, openai_model};
use serde::{Deserialize, Serialize};

use crate::cli::config::{ConfigOptions, MagicCliConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub model: Option<String>,
    pub embedding_model: Option<String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_endpoint: None,
            model: Some(openai_model::GPT_4O_MINI.to_string()),
            embedding_model: Some(openai_embedding_model::TEXT_EMBEDDING_ADA_002.to_string()),
        }
    }
}

impl ConfigOptions for OpenAiConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;

        let defaults = OpenAiConfig::default();
        if self.api_key.is_none() {
            populated = true;
            self.api_key = defaults.api_key;
        }
        if self.api_endpoint.is_none() {
            populated = true;
            self.api_endpoint = defaults.api_endpoint;
        }
        if self.model.is_none() {
            populated = true;
            self.model = defaults.model;
        }
        if self.embedding_model.is_none() {
            populated = true;
            self.embedding_model = defaults.embedding_model;
        }

        Ok(populated)
    }
}
