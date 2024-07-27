use orch::lm::anthropic_model;
use serde::{Deserialize, Serialize};

use crate::cli::config::{ConfigOptions, MagicCliConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: Some(anthropic_model::CLAUDE_3_5_SONNET.to_string()),
        }
    }
}

impl ConfigOptions for AnthropicConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;

        let defaults = AnthropicConfig::default();
        if self.api_key.is_none() {
            populated = true;
            self.api_key = defaults.api_key;
        }
        if self.model.is_none() {
            populated = true;
            self.model = defaults.model;
        }

        Ok(populated)
    }
}
