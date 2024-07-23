use serde::{Deserialize, Serialize};

use crate::cli::config::{ConfigOptions, MagicCliConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub embedding_model: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: Some("http://localhost:11434".to_string()),
            model: Some("codestral:latest".to_string()),
            embedding_model: Some("nomic-embed-text:latest".to_string()),
        }
    }
}

impl ConfigOptions for OllamaConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;
        let defaults = OllamaConfig::default();

        if self.base_url.is_none() {
            populated = true;
            self.base_url = defaults.base_url;
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
