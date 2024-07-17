use std::error::Error;

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

impl MagicCliSubcommand for SearchSubcommand {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let llm = MagicCliConfig::llm_from_config(&config)?;
        let cli_search = CliSearch::new(llm);
        let selected_command = cli_search.search_command(&self.prompt, self.index)?;

        let config = MagicCliConfig::load_config()?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&selected_command)?;
        Ok(())
    }
}
