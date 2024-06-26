use serde::{Deserialize, Serialize};
use simsimd::SpatialSimilarity;
use thiserror::Error;

use super::Llm;

#[derive(Debug, Error)]
pub enum SemanticSearchEngineError {
    #[error("Failed to generate embedding by LLM: {0}")]
    Embedding(String),
}

#[derive(Debug)]
pub struct SemanticSearchResult {
    pub id: usize,
    pub data: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HayStackItem {
    pub id: usize,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexItem {
    item: HayStackItem,
    embedding: Vec<f32>,
}

pub struct SemanticSearchEngine {
    llm: Box<dyn Llm>,
}

impl SemanticSearchEngine {
    pub fn new(llm: Box<dyn Llm>) -> Self {
        Self { llm }
    }

    pub fn top_k(&self, item: &str, index: Vec<IndexItem>, k: usize) -> Result<Vec<SemanticSearchResult>, SemanticSearchEngineError> {
        let needle_embedding = self
            .llm
            .generate_embedding(item)
            .map_err(|e| SemanticSearchEngineError::Embedding(e.to_string()))?;
        let mut similarities: Vec<SemanticSearchResult> = index
            .iter()
            .filter_map(|item| {
                let similarity = f32::cosine(item.embedding.as_slice(), needle_embedding.as_slice());
                similarity.map(|score| SemanticSearchResult {
                    id: item.item.id,
                    data: item.item.data.clone(),
                    score,
                })
            })
            .collect();

        similarities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(similarities.into_iter().rev().take(k).collect())
    }

    pub fn generate_index(&self, hay_stack: Vec<HayStackItem>) -> Result<Vec<IndexItem>, SemanticSearchEngineError> {
        let hay_stack_embeddings: Vec<IndexItem> = hay_stack
            .iter()
            .map(|item| {
                let embedding = self.llm.generate_embedding(&item.data).unwrap();
                IndexItem {
                    item: HayStackItem {
                        id: item.id,
                        data: item.data.clone(),
                    },
                    embedding,
                }
            })
            .collect();
        Ok(hay_stack_embeddings)
    }
}
