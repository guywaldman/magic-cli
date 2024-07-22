use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub embedding_model: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: Some("http://localhost:11434".to_string()),
            model: Some("codestral:latest".to_string()),
            embedding_model: Some("nomic-embed-text:latest".to_string()),
        }
    }
}
