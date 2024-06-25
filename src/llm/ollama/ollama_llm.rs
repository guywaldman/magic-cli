use thiserror::Error;

use crate::core::{Llm, LlmError};

use super::{
    config::OllamaConfig,
    models::{OllamaApiModelsMetadata, OllamaEmbeddingsRequest, OllamaEmbeddingsResponse, OllamaGenerateRequest, OllamaGenerateResponse},
};

#[derive(Debug, Clone)]
pub struct OllamaLocalLlm {
    config: OllamaConfig,
}

#[derive(Error, Debug)]
pub enum OllamaLocalLlmError {
    #[error("Unexpected response from API. Error: {0}")]
    Api(String),

    #[error("Unexpected error when parsing response from Ollama. Error: {0}")]
    Parsing(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Ollama API is not available. Please check if Ollama is running in the specified port. Error: {0}")]
    ApiUnavailable(String),
}

impl OllamaLocalLlm {
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
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, OllamaLocalLlmError> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/api/generate", self.config.base_url);
        let body = OllamaGenerateRequest {
            model: self.config.model.to_string(),
            prompt: prompt.to_string(),
            stream: Some(false),
            format: Some("json".to_string()),
            images: None,
            system: Some(system_prompt.to_string()),
            keep_alive: Some("5m".to_string()),
        };
        let response = client
            .post(url)
            .body(serde_json::to_string(&body).map_err(|e| OllamaLocalLlmError::Serialization(e.to_string()))?)
            .send()
            .map_err(|e| OllamaLocalLlmError::ApiUnavailable(e.to_string()))?;
        let body = response.text().map_err(|e| OllamaLocalLlmError::Api(e.to_string()))?;
        let response: OllamaGenerateResponse = serde_json::from_str(&body).map_err(|e| OllamaLocalLlmError::Parsing(e.to_string()))?;

        Ok(response.response)
    }

    /// Generates an embedding from the Ollama API.
    ///
    /// # Arguments
    /// * `prompt` - The item to generate an embedding for.
    ///
    /// # Returns
    ///
    /// A [Result] containing the embedding or an error if there was a problem.
    fn generate_embedding(&self, prompt: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/api/embeddings", self.config.base_url);
        let body = OllamaEmbeddingsRequest {
            model: self.config.embedding_model.to_string(),
            prompt: prompt.to_string(),
        };
        let response = client
            .post(url)
            .body(serde_json::to_string(&body).map_err(|e| OllamaLocalLlmError::Serialization(e.to_string()))?)
            .send()
            .map_err(|e| OllamaLocalLlmError::ApiUnavailable(e.to_string()))?;
        let body = response.text().map_err(|e| OllamaLocalLlmError::Api(e.to_string()))?;
        let response: OllamaEmbeddingsResponse = serde_json::from_str(&body).map_err(|e| OllamaLocalLlmError::Parsing(e.to_string()))?;

        Ok(response.embedding)
    }

    /// Lists the running models in the Ollama API.
    ///
    /// # Returns
    ///
    /// A [Result] containing the list of running models or an error if there was a problem.
    ///
    #[allow(dead_code)]
    pub(crate) fn list_running_models(&self) -> Result<OllamaApiModelsMetadata, OllamaLocalLlmError> {
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

    fn parse_models_response(response: &str) -> Result<OllamaApiModelsMetadata, OllamaLocalLlmError> {
        let models: OllamaApiModelsMetadata = serde_json::from_str(response).map_err(|e| OllamaLocalLlmError::Parsing(e.to_string()))?;
        Ok(models)
    }

    fn get_from_ollama_api(&self, url: &str) -> Result<String, OllamaLocalLlmError> {
        let url = format!("{}/{}", self.config.base_url, url);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .send()
            .map_err(|e| OllamaLocalLlmError::ApiUnavailable(e.to_string()))?;
        let response_text = response.text().map_err(|e| OllamaLocalLlmError::Api(e.to_string()))?;
        Ok(response_text)
    }
}

impl Llm for OllamaLocalLlm {
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, LlmError> {
        let response = self
            .generate(prompt, system_prompt)
            .map_err(|e| LlmError::TextGeneration(e.to_string()))?;
        Ok(response)
    }

    fn generate_embedding(&self, item: &str) -> Result<Vec<f32>, LlmError> {
        let response = self
            .generate_embedding(item)
            .map_err(|e| LlmError::EmbeddingGeneration(e.to_string()))?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::llm::ollama::models::{OllamaApiModelDetails, OllamaApiModelMetadata};

    use super::*;
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };

    #[test]
    fn test_list_models() {
        let mock_list_models_response = OllamaApiModelsMetadata {
            models: vec![OllamaApiModelMetadata {
                name: "mockstral:latest".to_string(),
                model: "mockstral:latest".to_string(),
                size: 12569170041,
                digest: "fcc0019dcee9947fe4298e23825eae643f4670e391f205f8c55a64c2068e9a22".to_string(),
                expires_at: None,
                details: OllamaApiModelDetails {
                    parent_model: "".to_string(),
                    format: "gguf".to_string(),
                    parameter_size: "7.2B".to_string(),
                    quantization_level: "Q4_0".to_string(),
                    family: "ollama".to_string(),
                },
            }],
        };

        let mock_server = MockServer::start();
        let mock_list_models_api = mock_server.mock(|when, then| {
            when.method(GET).path("/api/ps");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_list_models_response).unwrap());
        });

        let ollama_config = OllamaConfig {
            base_url: mock_server.base_url(),
            model: "mockstral:latest".to_string(),
            embedding_model: "mockembed:latest".to_string(),
        };

        let ollama = OllamaLocalLlm::new(ollama_config.clone());
        let running_models = ollama.list_running_models();
        mock_list_models_api.assert();
        assert!(running_models.is_ok());
        let running_models = running_models.unwrap();
        assert!(running_models.models.len() == 1);
        let model = running_models.models.first().unwrap();
        assert_eq!(model.name, mock_list_models_response.models.first().unwrap().name);
    }

    #[test]
    fn test_generate() {
        let mock_server = MockServer::start();
        let mock_generated_response = OllamaGenerateResponse {
            model: "mockstral:latest".to_string(),
            created_at: "2024-06-25T01:40:42.192756+00:00".to_string(),
            response: "Mock response".to_string(),
            total_duration: 12345,
        };
        let mock_generation_api = mock_server.mock(|when, then| {
            when.method(POST).path("/api/generate");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_generated_response).unwrap());
        });

        let ollama_config = OllamaConfig {
            base_url: mock_server.base_url(),
            model: "mockstral:latest".to_string(),
            embedding_model: "mockembed:latest".to_string(),
        };
        let ollama = OllamaLocalLlm::new(ollama_config.clone());

        let generation_response = ollama.generate("Mock prompt", "Mock system prompt");
        mock_generation_api.assert();
        assert!(generation_response.is_ok());
    }
}
