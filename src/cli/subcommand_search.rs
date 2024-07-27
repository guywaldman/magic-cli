use std::{error::Error, path::PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::Shell;

use super::{
    command::CliCommand,
    config::{ConfigOptions, MagicCliConfigError},
    search::{CliSearch, SearchOptions},
    subcommand::{MagicCliRunOptions, MagicCliSubcommand},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Whether to allow remote LLMs to embed shell history.
    pub allow_remote_llm: Option<bool>,

    /// Optional path to the shell history file, otherwise uses the default according to the user's system.
    pub shell_history: Option<PathBuf>,

    /// Optional path to the index directory, otherwise uses the default according to the user's system.
    pub index_dir: Option<PathBuf>,
}

impl ConfigOptions for SearchConfig {
    fn populate_defaults(&mut self) -> Result<bool, MagicCliConfigError> {
        let mut populated = false;
        let defaults = SearchConfig::default();

        if self.allow_remote_llm.is_none() {
            populated = true;
            self.allow_remote_llm = defaults.allow_remote_llm;
        }
        Ok(populated)
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            allow_remote_llm: Some(false),
            shell_history: None,
            index_dir: None,
        }
    }
}

pub struct SearchSubcommand {
    pub prompt: String,
    pub index: bool,
    pub reset_index: bool,
    pub output_only: bool,
}

#[async_trait]
impl MagicCliSubcommand for SearchSubcommand {
    async fn run(&self, options: MagicCliRunOptions) -> Result<(), Box<dyn Error>> {
        let config_mgr = &options.config;
        let config = &config_mgr.config;

        // Guard gainst using remote LLMs for embedding shell history.
        let Some(general_config) = config.general.clone() else {
            return Err(Box::new(MagicCliConfigError::MissingConfigKey("general".to_string())));
        };
        let Some(llm) = general_config.llm else {
            return Err(Box::new(MagicCliConfigError::MissingConfigKey("general.llm".to_string())));
        };
        if !llm.is_local() {
            let Some(search_config) = config.search.clone() else {
                return Err(Box::new(MagicCliConfigError::MissingConfigKey("search".to_string())));
            };
            let Some(allow_remote_llm) = search_config.allow_remote_llm else {
                return Err(Box::new(MagicCliConfigError::MissingConfigKey(
                    "search.allow_remote_llm".to_string(),
                )));
            };
            if !allow_remote_llm {
                return Err(Box::new(MagicCliConfigError::Configuration(
                    "Using remote LLM but `search.allow_remote_llm` is set to false. Set it to `true` if you are willing for remote LLM providers such as OpenAI to embed your shell history which may contains sensitive information.".to_string(),
                )));
            }
        }

        let lm = config_mgr.lm_from_config()?;
        let cli_search = CliSearch::new(dyn_clone::clone_box(&*lm));
        let Some(search_config) = config.search.clone() else {
            return Err(Box::new(MagicCliConfigError::MissingConfigKey("search".to_string())));
        };
        let shell_history_path = search_config
            .shell_history
            .unwrap_or_else(|| Shell::shell_history_path(None).unwrap());
        let index_dir_path = search_config
            .index_dir
            .unwrap_or_else(|| CliSearch::default_index_dir_path().unwrap());
        let search_options = SearchOptions {
            index: self.index,
            reset_index: self.reset_index,
            shell_history_path,
            index_dir_path,
            output_only: self.output_only,
        };
        let selected_command = cli_search.search_command(&self.prompt, &search_options).await;
        let selected_command = match selected_command {
            Ok(selected_command) => selected_command,
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        if self.output_only {
            // In "output only" mode, skip the interactive prompts and simply print the command.
            if let Some(selected_command) = selected_command {
                println!("{}", selected_command);
            }
            return Ok(());
        }

        let Some(selected_command) = selected_command else {
            return Ok(());
        };

        let Some(suggest_config) = config.suggest.clone() else {
            return Err(Box::new(MagicCliConfigError::MissingConfigKey("suggest".to_string())));
        };

        CliCommand::new(suggest_config).suggest_user_action_on_command(&selected_command)?;

        Ok(())
    }
}
