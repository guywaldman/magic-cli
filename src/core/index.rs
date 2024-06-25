use std::{path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::{HayStackItem, IndexItem, Llm, SemanticSearchEngine, SemanticSearchEngineError};

#[derive(Debug, Error)]
pub(crate) enum IndexError {
    #[error("Failed to generate index: {0}")]
    IndexGeneration(#[from] SemanticSearchEngineError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Marshalling error: {0}")]
    Marshalling(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexData {
    items: Vec<IndexItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub last_index_time: Option<SystemTime>,
}

pub(crate) struct IndexEngine<L: Llm> {
    semantic_search_engine: SemanticSearchEngine<L>,
    index_path: PathBuf,
    index_metadata_path: PathBuf,
}

impl<L: Llm> IndexEngine<L> {
    pub fn new(semantic_search_engine: SemanticSearchEngine<L>, index_path: PathBuf, index_metadata_path: PathBuf) -> Self {
        Self {
            semantic_search_engine,
            index_path,
            index_metadata_path,
        }
    }

    pub fn store_index(&self, hay_stack: Vec<HayStackItem>) -> Result<(), IndexError> {
        let index = self
            .semantic_search_engine
            .generate_index(hay_stack)
            .map_err(IndexError::IndexGeneration)?;
        let index_data = IndexData { items: index };
        let serialized_index = serde_json::to_string(&index_data).map_err(IndexError::Marshalling)?;
        std::fs::write(&self.index_path, serialized_index).map_err(IndexError::IoError)?;
        self.update_index_metadata()?;
        Ok(())
    }

    pub fn load_index(&self) -> Result<Vec<IndexItem>, IndexError> {
        let index_content = std::fs::read_to_string(&self.index_path).map_err(IndexError::IoError)?;
        let index_data: IndexData = serde_json::from_str(&index_content).map_err(IndexError::Marshalling)?;
        Ok(index_data.items)
    }

    pub fn load_index_metadata(&self) -> Result<IndexMetadata, IndexError> {
        if !self.index_metadata_path.exists() {
            return Ok(IndexMetadata { last_index_time: None });
        }
        let index_metadata_content = std::fs::read_to_string(&self.index_metadata_path).map_err(IndexError::IoError)?;
        let index_metadata_data: IndexMetadata = serde_json::from_str(&index_metadata_content).map_err(IndexError::Marshalling)?;
        Ok(index_metadata_data)
    }

    fn update_index_metadata(&self) -> Result<(), IndexError> {
        let index_metadata = IndexMetadata {
            last_index_time: Some(SystemTime::now()),
        };
        let serialized_index_metadata = serde_json::to_string(&index_metadata).map_err(IndexError::Marshalling)?;
        std::fs::write(&self.index_metadata_path, serialized_index_metadata).map_err(IndexError::IoError)?;
        Ok(())
    }
}
