use thiserror::Error;

use super::{
    config::OllamaConfig,
    models::{OllamaGenerateRequest, OllamaGenerateResponse},
};

#[derive(Debug)]
pub struct OllamaApiClient {
    config: OllamaConfig,
}

#[derive(Error, Debug)]
pub enum OllamaApiClientError {
    #[error("Unexpected response from API. Error: {0}")]
    ApiError(String),

    #[error("Ollama API is not available. Please check if Ollama is running in the specified port. Error: {0}")]
    ApiUnavailable(String),
}

impl OllamaApiClient {
    pub fn new(config: OllamaConfig) -> Self {
        Self { config }
    }

    // pub fn list_local_models(&self) -> Result<OllamaApiModelsMetadata, OllamaApiClientError> {
    //     let client = reqwest::blocking::Client::new();
    //     let url = format!("{}/api/tags", self.config.base_url);
    //     let response = client
    //         .get(url)
    //         .send()
    //         .map_err(|e| OllamaApiClientError::ApiUnavailable(e.to_string()))?;
    //     let body = response.text().map_err(|e| OllamaApiClientError::ApiError(e.to_string()))?;
    //     let models: OllamaApiModelsMetadata = serde_json::from_str(&body).map_err(|e| OllamaApiClientError::ApiError(e.to_string()))?;
    //     Ok(models)
    // }

    pub fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, OllamaApiClientError> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/api/generate", self.config.base_url);
        let body = OllamaGenerateRequest {
            model: self.config.active_model_id.clone(),
            prompt: prompt.to_string(),
            stream: Some(false),
            format: Some("json".to_string()),
            images: None,
            system: Some(system_prompt.to_string()),
            keep_alive: Some("5m".to_string()),
        };
        let response = client
            .post(url)
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .map_err(|e| OllamaApiClientError::ApiUnavailable(e.to_string()))?;
        let body = response.text().map_err(|e| OllamaApiClientError::ApiError(e.to_string()))?;
        let response: OllamaGenerateResponse = serde_json::from_str(&body).map_err(|e| OllamaApiClientError::ApiError(e.to_string()))?;
        Ok(response.response)
    }
}
