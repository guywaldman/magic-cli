use chrono::Duration;
use colored::Colorize;
use inquire::{list_option::ListOption, InquireError, Select};
use orch::lm::LanguageModel;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::SystemTime,
};
use thiserror::Error;

use crate::core::{
    HayStackItem, IndexEngine, IndexError, IndexMetadata, SemanticSearchEngine, SemanticSearchEngineError, Shell, ShellError,
};

use super::config::{MagicCliConfigError, MagicCliConfigManager};

#[derive(Debug, Error)]
pub(crate) enum CliSearchError {
    #[error("Error during indexing or loading index: {0}")]
    Indexing(#[from] IndexError),

    #[error("Error during loading of configuration: {0}")]
    Config(#[from] MagicCliConfigError),

    #[error("Error from shell interaction: {0}")]
    Shell(#[from] ShellError),

    #[error("Semantic search error: {0}")]
    SemanticSearch(#[from] SemanticSearchEngineError),

    #[error("Command not selected: {0}")]
    CommandNotSelected(#[from] InquireError),

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("No results found")]
    NoResultsFound,
}

pub(crate) struct CliSearch {
    lm: Box<dyn LanguageModel>,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub index: bool,
    pub reset_index: bool,
    pub index_dir_path: PathBuf,
    pub shell_history_path: PathBuf,
    pub output_only: bool,
}

impl CliSearch {
    pub fn new(llm: Box<dyn LanguageModel>) -> Self {
        Self { lm: llm }
    }

    pub async fn search_command(&self, prompt: &str, options: &SearchOptions) -> Result<Option<String>, CliSearchError> {
        let SearchOptions {
            index,
            reset_index,
            index_dir_path,
            shell_history_path,
            output_only,
        } = options.clone();

        if reset_index {
            if !output_only {
                println!("{}", "Resetting index...".yellow());
            }
            self.reset_index(&index_dir_path)?;
            if !output_only {
                println!("{}", "Index reset successfully.".green());
            }
        }

        if index {
            // Index shell history if needed and `index` is set to true.
            self.index_shell_history(&shell_history_path, &index_dir_path, output_only).await?;
        }

        let index_path = Self::index_path(&index_dir_path)?;
        let index_metadata_path = Self::index_metadata_path(&index_dir_path)?;

        let index_engine = IndexEngine::new(
            SemanticSearchEngine::new(dyn_clone::clone_box(&*self.lm)),
            index_path,
            index_metadata_path,
        );

        let index = index_engine.load_index()?;
        let semantic_search_engine = SemanticSearchEngine::new(dyn_clone::clone_box(&*self.lm));
        let semantic_search_results = semantic_search_engine.top_k(prompt, index, 10).await?;

        if semantic_search_results.is_empty() && !output_only {
            println!("{}", "No relevant results found.".yellow().bold());
            return Ok(None);
        }

        let options = semantic_search_results
            .iter()
            .map(|result| ListOption::new(result.id, result.data.clone()))
            .collect::<Vec<_>>();
        if output_only {
            // In "output only" mode, skip the interactive prompts and simply print the command.
            let Some(top_result) = semantic_search_results.first() else {
                return Err(CliSearchError::NoResultsFound);
            };
            return Ok(Some(top_result.data.clone()));
        }

        let selected_command = Select::new(
            "Select a command:",
            options,
            // &semantic_search_results.iter().map(|result| result.data.clone()).collect::<Vec<_>>(),
        )
        .prompt();

        selected_command
            .map(|op| Some(op.to_string()))
            .map_err(CliSearchError::CommandNotSelected)
    }

    async fn index_shell_history(&self, shell_history_path: &Path, index_dir_path: &Path, output_only: bool) -> Result<(), CliSearchError> {
        let index_path = Self::index_path(index_dir_path)?;
        let index_metadata_path = Self::index_metadata_path(index_dir_path)?;

        let index_engine = IndexEngine::new(
            SemanticSearchEngine::new(dyn_clone::clone_box(&*self.lm)),
            index_path,
            index_metadata_path,
        );
        let index_metadata = index_engine.load_index_metadata()?;
        let should_update_index = match index_metadata {
            IndexMetadata {
                last_index_time: Some(last_index_time),
            } => {
                let now = SystemTime::now();
                let duration = now.duration_since(last_index_time).unwrap();
                duration.as_secs() > Duration::hours(24).num_seconds() as u64
            }
            IndexMetadata { last_index_time: None } => true,
        };
        if should_update_index {
            if !output_only {
                println!("{}", "Updating index (this may take a few moments)...".yellow());
            }
            let shell_history = Shell::get_shell_history(shell_history_path)?;
            let mut hay_stack_map = HashSet::new();
            for item in shell_history.iter() {
                if !hay_stack_map.contains(item) {
                    hay_stack_map.insert(item);
                }
            }
            let hay_stack = hay_stack_map
                .into_iter()
                .enumerate()
                .map(|(i, item)| HayStackItem { id: i, data: item.clone() })
                .collect();
            index_engine.store_index(hay_stack).await?;
            if !output_only {
                println!("{}", "Index updated successfully.".green());
            }
        }
        Ok(())
    }

    fn reset_index(&self, index_dir_path: &Path) -> Result<(), CliSearchError> {
        let index_path = Self::index_path(index_dir_path)?;
        let index_metadata_path = Self::index_metadata_path(index_dir_path)?;
        std::fs::remove_file(index_path).map_err(CliSearchError::IoError)?;
        std::fs::remove_file(index_metadata_path).map_err(CliSearchError::IoError)?;
        Ok(())
    }

    pub fn default_index_dir_path() -> Result<PathBuf, CliSearchError> {
        let index_dir_path = MagicCliConfigManager::get_config_default_dir_path()?.join("index");
        if !index_dir_path.exists() {
            std::fs::create_dir_all(&index_dir_path).unwrap();
        }
        Ok(index_dir_path)
    }

    fn index_path(index_dir_path: &Path) -> Result<PathBuf, CliSearchError> {
        let index_path = index_dir_path.join("index.json");
        Ok(index_path)
    }

    fn index_metadata_path(index_dir_path: &Path) -> Result<PathBuf, CliSearchError> {
        let index_metadata_path = index_dir_path.join("index_metadata.json");
        Ok(index_metadata_path)
    }
}
