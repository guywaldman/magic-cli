use std::error::Error;

use crate::core::SuggestionEngine;

use super::{command::CliCommand, config::MagicCliConfig, subcommand::MagicCliSubcommand};

pub struct SuggestSubcommand {
    prompt: String,
}

impl SuggestSubcommand {
    pub fn new(prompt: String) -> Self {
        Self { prompt }
    }
}

impl MagicCliSubcommand for SuggestSubcommand {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let llm = MagicCliConfig::llm_from_config(&config)?;
        let explain_subcommand = SuggestionEngine::new(llm);
        let command = explain_subcommand.suggest_command(&self.prompt)?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&command)?;
        Ok(())
    }
}
