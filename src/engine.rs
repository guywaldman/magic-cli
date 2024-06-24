use serde::Deserialize;
use thiserror::Error;

use crate::ollama::client::OllamaApiClient;

#[derive(Debug)]
pub struct Engine {
    ollama_client: OllamaApiClient,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub fn new(ollama_client: OllamaApiClient) -> Self {
        Self { ollama_client }
    }

    pub fn suggest_command(&self, prompt: &str) -> Result<SuggestedCommand, EngineError> {
        const SYSTEM_PROMPT: &str = "
        You are a an assistant that provides suggestions for a command to
        run on a Linux machine that satisfies the user's request.
        Please only provide a JSON which contains the fields 'command' for the command,
        'explanation' for a very short explanation about the command or notes you may have,
        and nothing else.
        For the command, if there are arguments, use the format '<argument_name>', for example 'kubectl logs -n <namespace> <pod-name>'.
        ";

        let response = self.ollama_client.generate(prompt, SYSTEM_PROMPT).unwrap();
        let parsed_response = serde_json::from_str(&response).unwrap();
        Ok(parsed_response)
    }

    pub fn suggest_command_with_revision(
        &self,
        previous_command: &SuggestedCommand,
        prompt: &str,
    ) -> Result<SuggestedCommand, EngineError> {
        const SYSTEM_PROMPT: &str = "
        You are a an assistant that provides suggestions for a command to
        run on a Linux machine that satisfies the user's request.
        You will receive a command that you suggested before and a prompt from the user to revise it.
        Please only provide a JSON which contains the fields 'command' for the command,
        'explanation' for a very short explanation about the command or notes you may have,
        and nothing else.
        For the command, if there are arguments, use the format '<argument_name>', for example 'kubectl logs -n <namespace> <pod-name>'.
        ";

        let prompt = format!(
            "
            The previous command was: `{}` (explanation: '{}')
            Please revise the command to satisfy the user's request.
            The revision prompt is: '{}'
        ",
            previous_command.command, previous_command.explanation, prompt
        );

        let response = self.ollama_client.generate(&prompt, SYSTEM_PROMPT).unwrap();
        let parsed_response = serde_json::from_str(&response).unwrap();
        Ok(parsed_response)
    }
}
