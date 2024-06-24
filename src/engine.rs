use serde::Deserialize;
use thiserror::Error;

use crate::ollama::client::OllamaApiClient;

#[derive(Debug)]
pub struct Engine {
    ollama_client: OllamaApiClient,
}

#[derive(Debug, Deserialize)]
pub struct SuggestedCommand {
    pub command: String,
    pub explanation: String,
}

#[derive(Error, Debug)]
pub enum EngineError {
    // #[error("General error: {0}")]
    // GeneralError(String),
}

impl Engine {
    pub fn new(client: OllamaApiClient) -> Self {
        Self { ollama_client: client }
    }

    pub fn suggest_command(&self, prompt: &str) -> Result<SuggestedCommand, EngineError> {
        const SYSTEM_PROMPT: &str = "
        You are a an assistant that provides suggestions for a command to
        run on a Linux machine that satisfies the user's request.
        Please only provide a JSON which contains the fields 'command' for the command,
        'explanation' for a very short explanation about the command or notes you may have,
        and nothing else.
        ";

        let response = self.ollama_client.generate(prompt, SYSTEM_PROMPT).unwrap();
        let parsed_response = serde_json::from_str(&response).unwrap();
        Ok(parsed_response)
    }
}
