use openai_api_rs::v1::{api::Client, chat_completion, embedding};

use crate::core::{Llm, LlmError};

use super::config::OpenAiConfig;

#[derive(Debug, Clone)]
pub struct OpenAiLlm {
    config: OpenAiConfig,
}

impl OpenAiLlm {
    pub fn new(config: OpenAiConfig) -> Self {
        Self { config }
    }

    fn api_key(&self) -> Result<String, LlmError> {
        self.config
            .api_key
            .clone()
            .ok_or_else(|| LlmError::Configuration("API key not set".to_string()))
    }

    fn model(&self) -> Result<String, LlmError> {
        self.config
            .model
            .clone()
            .ok_or_else(|| LlmError::Configuration("Model not set".to_string()))
    }

    fn embedding_model(&self) -> Result<String, LlmError> {
        self.config
            .embedding_model
            .clone()
            .ok_or_else(|| LlmError::Configuration("Embedding model not set".to_string()))
    }
}

impl Llm for OpenAiLlm {
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, LlmError> {
        let api_key = self.api_key()?;
        let model = self.model()?;

        let client = Client::new(api_key);
        let chat_completion_request = chat_completion::ChatCompletionRequest::new(
            model,
            vec![
                chat_completion::ChatCompletionMessage {
                    name: None,
                    role: chat_completion::MessageRole::system,
                    content: chat_completion::Content::Text(system_prompt.to_string()),
                },
                chat_completion::ChatCompletionMessage {
                    name: None,
                    role: chat_completion::MessageRole::user,
                    content: chat_completion::Content::Text(prompt.to_string()),
                },
            ],
        );
        let response = client
            .chat_completion(chat_completion_request)
            .map_err(|e| LlmError::TextGeneration(e.to_string()))?;
        let response_content = response
            .choices
            .first()
            .ok_or_else(|| LlmError::TextGeneration("Invalid response from OpenAI API".to_string()))?
            .message
            .content
            .clone();
        match response_content {
            Some(content) => Ok(content),
            None => Err(LlmError::TextGeneration("Invalid response from OpenAI API".to_string())),
        }
    }

    fn generate_embedding(&self, item: &str) -> Result<Vec<f32>, LlmError> {
        let api_key = self.api_key()?;
        let embedding_model = self.embedding_model()?;

        let client = Client::new(api_key);
        let embedding_request = embedding::EmbeddingRequest::new(embedding_model, item.to_string());
        let response = client
            .embedding(embedding_request)
            .map_err(|e| LlmError::EmbeddingGeneration(e.to_string()))?;
        let item_embedding = response
            .data
            .first()
            .ok_or_else(|| LlmError::EmbeddingGeneration("Invalid response from OpenAI API".to_string()))?
            .embedding
            .clone();
        Ok(item_embedding)
    }
}
