use std::error::Error;

use crate::core::SuggestionEngine;

use super::{command::CliCommand, config::MagicCliConfig};

pub struct SuggestSubcommand;

impl SuggestSubcommand {
    pub fn run(prompt: &str) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let llm = MagicCliConfig::llm_from_config(&config)?;
        let explain_subcommand = SuggestionEngine::new(llm);
        let command = explain_subcommand.suggest_command(prompt)?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&command)?;
        Ok(())
    }
}
