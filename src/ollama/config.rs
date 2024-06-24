#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub active_model_id: String,
}

impl OllamaConfig {
    // fn new(base_url: &str, active_model_id: &str) -> Self {
    //     Self {
    //         base_url: base_url.to_string(),
    //         active_model_id: active_model_id.to_string(),
    //     }
    // }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            active_model_id: "codestral:latest".to_string(),
        }
    }
}
