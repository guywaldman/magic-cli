use thiserror::Error;

use super::{
    config::OllamaConfig,
    models::{OllamaApiModelsMetadata, OllamaGenerateRequest, OllamaGenerateResponse},
};

#[derive(Debug, Clone)]
pub struct OllamaApiClient {
    config: OllamaConfig,
}

#[derive(Error, Debug)]
pub enum OllamaApiClientError {
    #[error("Unexpected response from API. Error: {0}")]
    ApiError(String),

    #[error("Unexpected error when parsing response from Ollama. Error: {0}")]
    ParsingError(String),

    #[error("Ollama API is not available. Please check if Ollama is running in the specified port. Error: {0}")]
    ApiUnavailable(String),
}

impl OllamaApiClient {
    pub fn new(config: OllamaConfig) -> Self {
        Self { config }
    }

    /// Generates a response from the Ollama API.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to generate a response for.
    /// * `system_prompt` - The system prompt to use for the generation.
    ///
    /// # Returns
    ///
    /// A [Result] containing the response from the Ollama API or an error if there was a problem.
    ///
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

    /// Lists the running models in the Ollama API.
    ///
    /// # Returns
    ///
    /// A [Result] containing the list of running models or an error if there was a problem.
    ///
    pub fn list_running_models(&self) -> Result<OllamaApiModelsMetadata, OllamaApiClientError> {
        let response = self.get_from_ollama_api("api/ps")?;
        let parsed_response = Self::parse_models_response(&response)?;
        Ok(parsed_response)
    }

    // /// Lists the local models in the Ollama API.
    // ///
    // /// # Returns
    // ///     
    // /// A [Result] containing the list of local models or an error if there was a problem.
    // pub fn list_local_models(&self) -> Result<OllamaApiModelsMetadata, OllamaApiClientError> {
    //     let response = self.get_from_ollama_api("api/tags")?;
    //     let parsed_response = Self::parse_models_response(&response)?;
    //     Ok(parsed_response)
    // }

    fn parse_models_response(response: &str) -> Result<OllamaApiModelsMetadata, OllamaApiClientError> {
        let models: OllamaApiModelsMetadata =
            serde_json::from_str(response).map_err(|e| OllamaApiClientError::ParsingError(e.to_string()))?;
        Ok(models)
    }

    fn get_from_ollama_api(&self, url: &str) -> Result<String, OllamaApiClientError> {
        let url = format!("{}/{}", self.config.base_url, url);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .send()
            .map_err(|e| OllamaApiClientError::ApiUnavailable(e.to_string()))?;
        let response_text = response.text().map_err(|e| OllamaApiClientError::ApiError(e.to_string()))?;
        Ok(response_text)
    }
}
