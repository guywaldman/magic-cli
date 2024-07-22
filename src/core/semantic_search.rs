use std::sync::Arc;

use orch::lm::LanguageModel;
use serde::{Deserialize, Serialize};
use simsimd::SpatialSimilarity;
use thiserror::Error;
use tokio::{sync::Mutex, task::JoinSet};

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
    lm: Arc<Mutex<Box<dyn LanguageModel>>>,
}

impl SemanticSearchEngine {
    const SIMILARITY_THRESHOLD: f64 = 0.2;

    pub fn new(lm: Box<dyn LanguageModel>) -> Self {
        Self {
            lm: Arc::new(Mutex::new(lm)),
        }
    }

    pub async fn top_k(&self, item: &str, index: Vec<IndexItem>, k: usize) -> Result<Vec<SemanticSearchResult>, SemanticSearchEngineError> {
        let needle_embedding = self
            .lm
            .lock()
            .await
            .generate_embedding(item)
            .await
            .map_err(|e| SemanticSearchEngineError::Embedding(e.to_string()))?;
        let mut similarities: Vec<SemanticSearchResult> = index
            .iter()
            .filter_map(|item| {
                let similarity = f32::cosine(item.embedding.as_slice(), needle_embedding.as_slice());

                if similarity.map(|score| score < Self::SIMILARITY_THRESHOLD).unwrap_or(false) {
                    return None;
                }

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

    pub async fn generate_index(&self, hay_stack: Vec<HayStackItem>) -> Result<Vec<IndexItem>, SemanticSearchEngineError> {
        let mut js: JoinSet<(Vec<f32>, usize)> = JoinSet::new();
        let data_items = hay_stack.iter().map(|item| (item.data.clone(), item.id)).collect::<Vec<_>>();
        let data_items_clone = data_items.clone();
        for item in data_items {
            let lm = self.lm.clone();
            js.spawn(async move {
                let embedding = {
                    let lm_locked = lm.lock().await;
                    lm_locked.generate_embedding(&item.0).await.unwrap()
                };
                (embedding, item.1)
            });
        }
        let mut embeddings: Vec<IndexItem> = Vec::new();
        while let Some(result) = js.join_next().await {
            match result {
                Ok(res) => {
                    embeddings.push(IndexItem {
                        item: HayStackItem {
                            id: data_items_clone[res.1].1,
                            data: data_items_clone[res.1].0.clone(),
                        },
                        embedding: res.0,
                    });
                }
                Err(e) => return Err(SemanticSearchEngineError::Embedding(e.to_string())),
            }
        }
        Ok(embeddings)
    }
}
