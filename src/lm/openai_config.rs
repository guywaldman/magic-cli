use orch::lm::{openai_embedding_model, openai_model};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub embedding_model: Option<String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: Some(openai_model::GPT_4O_MINI.to_string()),
            embedding_model: Some(openai_embedding_model::TEXT_EMBEDDING_ADA_002.to_string()),
        }
    }
}
