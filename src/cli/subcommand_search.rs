use std::error::Error;

use async_trait::async_trait;

use super::{command::CliCommand, config::MagicCliConfig, search::CliSearch, subcommand::MagicCliSubcommand};

pub struct SearchSubcommand {
    prompt: String,
    index: bool,
}

impl SearchSubcommand {
    pub fn new(prompt: String, index: bool) -> Self {
        Self { prompt, index }
    }
}

#[async_trait]
impl MagicCliSubcommand for SearchSubcommand {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let lm = MagicCliConfig::lm_from_config(&config)?;
        let cli_search = CliSearch::new(dyn_clone::clone_box(&*lm));
        let selected_command = cli_search.search_command(&self.prompt, self.index).await?;

        let config = MagicCliConfig::load_config()?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&selected_command)?;
        Ok(())
    }
}
