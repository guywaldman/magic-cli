use std::error::Error;

use super::{command::CliCommand, config::MagicCliConfig, search::CliSearch};

pub struct SearchSubcommand;

impl SearchSubcommand {
    pub fn run(prompt: &str, index: bool) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let llm = MagicCliConfig::llm_from_config(&config)?;
        let cli_search = CliSearch::new(llm);
        let selected_command = cli_search.search_command(prompt, index)?;

        let config = MagicCliConfig::load_config()?;
        CliCommand::new(config.suggest).suggest_user_action_on_command(&selected_command)?;
        Ok(())
    }
}
