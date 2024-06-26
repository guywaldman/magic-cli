use chrono::Duration;
use colored::Colorize;
use inquire::{list_option::ListOption, InquireError, Select};
use std::{collections::HashSet, time::SystemTime};
use thiserror::Error;

use crate::core::{
    HayStackItem, IndexEngine, IndexError, IndexMetadata, Llm, SemanticSearchEngine, SemanticSearchEngineError, Shell, ShellError,
};

use super::config::{MagicCliConfig, MagicCliConfigError};

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
}

pub(crate) struct CliSearch {
    llm: Box<dyn Llm>,
}

impl CliSearch {
    pub fn new(llm: Box<dyn Llm>) -> Self {
        Self { llm }
    }

    pub fn search_command(&self, prompt: &str, index: bool) -> Result<String, CliSearchError> {
        let index_dir_path = MagicCliConfig::get_config_dir_path()?.join("index");
        if !index_dir_path.exists() {
            std::fs::create_dir_all(&index_dir_path).unwrap();
        }
        let index_path = index_dir_path.join("index.json");
        let index_metadata_path = index_dir_path.join("index_metadata.json");
        let index_engine = IndexEngine::new(
            SemanticSearchEngine::new(dyn_clone::clone_box(&*self.llm)),
            index_path,
            index_metadata_path,
        );
        let index_metadata = index_engine.load_index_metadata()?;
        let should_update_index = index
            || match index_metadata {
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
            println!("{}", "Updating index (this may take a few moments)...".yellow());
            let shell_history = Shell::get_shell_history()?;
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
            index_engine.store_index(hay_stack)?;
            println!("{}", "Index updated successfully.".green());
        }

        let index = index_engine.load_index()?;
        let semantic_search_engine = SemanticSearchEngine::new(dyn_clone::clone_box(&*self.llm));
        let semantic_search_results = semantic_search_engine.top_k(prompt, index, 10)?;

        let options = semantic_search_results
            .iter()
            .map(|result| ListOption::new(result.id, result.data.clone()))
            .collect::<Vec<_>>();
        let selected_command = Select::new(
            "Select a command:",
            options,
            // &semantic_search_results.iter().map(|result| result.data.clone()).collect::<Vec<_>>(),
        )
        .prompt();

        selected_command
            .map(|op| op.to_string())
            .map_err(CliSearchError::CommandNotSelected)
    }
}
