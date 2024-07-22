use std::error::Error;

use async_trait::async_trait;

use crate::core::SuggestionEngine;

use super::{
    command::CliCommand,
    config::MagicCliConfigError,
    subcommand::{MagicCliRunOptions, MagicCliSubcommand},
};

pub struct SuggestSubcommandArguments {
    pub prompt: String,
    pub output_only: bool,
}

pub struct SuggestSubcommand {
    args: SuggestSubcommandArguments,
}

impl SuggestSubcommand {
    pub fn new(args: SuggestSubcommandArguments) -> Self {
        Self { args }
    }
}

#[async_trait]
impl MagicCliSubcommand for SuggestSubcommand {
    async fn run(&self, options: MagicCliRunOptions) -> Result<(), Box<dyn Error>> {
        let lm = options.config.lm_from_config()?;
        let explain_subcommand = SuggestionEngine::new(lm.as_ref());
        let command = explain_subcommand.suggest_command(&self.args.prompt, self.args.output_only).await?;
        if !self.args.output_only {
            let config_values = options.config.load_config()?;
            let Some(suggest_config) = config_values.suggest.clone() else {
                return Err(Box::new(MagicCliConfigError::MissingConfigKey("suggest".to_string())));
            };
            CliCommand::new(suggest_config).suggest_user_action_on_command(&command)?;
        } else {
            // In "output only" mode, skip the interactive prompts and simply print the command.
            println!("{}", command);
        }
        Ok(())
    }
}
