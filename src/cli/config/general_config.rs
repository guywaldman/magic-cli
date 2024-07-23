use orch::lm::LanguageModelProvider;
use serde::{Deserialize, Serialize};

use super::{ConfigOptions, MagicCliConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The LLM provider to use for generating responses.
    pub llm: Option<LanguageModelProvider>,

    /// Whether to allow access to the shell history.
    pub access_to_shell_history: Option<bool>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            llm: Some(LanguageModelProvider::Ollama),
            access_to_shell_history: Some(true),
        }
    }
}

impl ConfigOptions for GeneralConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;
        let defaults = GeneralConfig::default();

        if self.llm.is_none() {
            populated = true;
            self.llm = defaults.llm;
        }
        if self.access_to_shell_history.is_none() {
            populated = true;
            self.access_to_shell_history = defaults.access_to_shell_history;
        }
        Ok(populated)
    }
}
