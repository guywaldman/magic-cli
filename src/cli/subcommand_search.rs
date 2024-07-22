use std::error::Error;

use async_trait::async_trait;

use super::{
    command::CliCommand,
    config::MagicCliConfigError,
    search::CliSearch,
    subcommand::{MagicCliRunOptions, MagicCliSubcommand},
};

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
    async fn run(&self, options: MagicCliRunOptions) -> Result<(), Box<dyn Error>> {
        let config = options.config;
        let lm = config.lm_from_config()?;
        let cli_search = CliSearch::new(dyn_clone::clone_box(&*lm));
        let selected_command = cli_search.search_command(&self.prompt, self.index).await?;
        let config_values = config.load_config()?;
        let Some(suggest_config) = config_values.suggest.clone() else {
            return Err(Box::new(MagicCliConfigError::MissingConfigKey("suggest".to_string())));
        };
        CliCommand::new(suggest_config).suggest_user_action_on_command(&selected_command)?;
        Ok(())
    }
}
