use std::error::Error;

use async_trait::async_trait;

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

#[async_trait]
impl MagicCliSubcommand for SuggestSubcommand {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let lm = MagicCliConfig::lm_from_config(&config)?;
        let explain_subcommand = SuggestionEngine::new(lm.as_ref());
        let command = explain_subcommand.suggest_command(&self.prompt).await?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&command)?;
        Ok(())
    }
}
