use colored::Colorize;
use inquire::Text;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::llm::Llm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestMode {
    #[serde(rename = "clipboard")]
    Clipboard,
    #[serde(rename = "unsafe-execution")]
    Execution,
}

#[derive(Error, Debug)]
pub enum SuggestModeError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

impl TryFrom<&str> for SuggestMode {
    type Error = SuggestModeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "clipboard" => Ok(SuggestMode::Clipboard),
            "unsafe-execution" => Ok(SuggestMode::Execution),
            _ => Err(SuggestModeError::InvalidValue(value.to_string())),
        }
    }
}

impl Default for SuggestMode {
    fn default() -> Self {
        Self::Clipboard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestConfig {
    pub mode: SuggestMode,
    pub add_to_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedCommand {
    pub command: String,
    pub explanation: String,
}

#[derive(Error, Debug)]
pub enum SuggestionEngineError {
    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Serialization or deserialization error: {0}")]
    Serialization(String),
}

pub struct SuggestionEngine {
    llm: Box<dyn Llm>,
}

impl SuggestionEngine {
    pub fn new(llm: Box<dyn Llm>) -> Self {
        Self { llm }
    }

    /// Suggests a command based on the given prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to use for generating the command suggestion.
    ///
    /// # Returns
    ///
    /// A [Result] containing the generated command suggestion or an error if there was a problem.
    ///
    // TODO: Move all CLI-related logic to the `cli` module.
    pub fn suggest_command(&self, prompt: &str) -> Result<String, SuggestionEngineError> {
        println!(
            "{} {}",
            "Generating suggested command for prompt".dimmed(),
            format!("\"{}\"...", prompt).dimmed(),
        );
        println!();

        let suggested_command = self
            .generate_suggested_command(prompt)
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;

        println!("{:>18}: {}", "Suggested command".dimmed(), suggested_command.command.blue().bold());
        println!("{:>18}: {}", "Explanation".dimmed(), suggested_command.explanation.italic());
        println!();

        let mut command = suggested_command.command.clone();
        loop {
            let revise_command = Text::new("Provide instructions on how to revise the command (leave empty to skip)").prompt();
            match revise_command {
                Ok(revision_prompt) => {
                    if revision_prompt.trim().is_empty() {
                        break;
                    }
                    let revision_respose = self.generate_suggested_command_with_revision(&suggested_command, &revision_prompt)?;
                    let revised_command = revision_respose.command;
                    command.clone_from(&revised_command);
                    println!("{} {}", "Suggested command:".dimmed(), revised_command.blue().bold());
                    println!();
                }
                Err(e) => {
                    println!("{}", e.to_string().red());
                    break;
                }
            };
        }
        println!();

        command = self.populate_placeholders_in_command(&command);

        Ok(command)
    }

    pub(crate) fn generate_suggested_command(&self, prompt: &str) -> Result<SuggestedCommand, SuggestionEngineError> {
        const SYSTEM_PROMPT: &str = "
        You are a an assistant that provides suggestions for a command to
        run on a Linux machine that satisfies the user's request.
        Please only provide a JSON which contains the fields 'command' for the command,
        'explanation' for a very short explanation about the command or notes you may have,
        and nothing else.
        For the command, if there are arguments, use the format '<argument_name>', for example 'kubectl logs -n <namespace> <pod-name>'.
        Remember to format the response as JSON.
        ";

        let response = self
            .llm
            .generate(prompt, SYSTEM_PROMPT)
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;
        if !response.trim().starts_with('{') || !response.trim().ends_with('}') {
            return Err(SuggestionEngineError::Generation(
                "Response from LLM is not in JSON format".to_string(),
            ));
        }
        let parsed_response = serde_json::from_str(&response).map_err(|e| SuggestionEngineError::Serialization(e.to_string()))?;
        Ok(parsed_response)
    }

    pub(crate) fn generate_suggested_command_with_revision(
        &self,
        previous_command: &SuggestedCommand,
        prompt: &str,
    ) -> Result<SuggestedCommand, SuggestionEngineError> {
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

        let response = self
            .llm
            .generate(&prompt, SYSTEM_PROMPT)
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;
        let parsed_response = serde_json::from_str(&response).map_err(|e| SuggestionEngineError::Serialization(e.to_string()))?;
        Ok(parsed_response)
    }

    fn populate_placeholders_in_command(&self, command: &str) -> String {
        let mut replaced_command = command.to_string();
        let placeholders = replaced_command
            .split_whitespace()
            .filter_map(|s| {
                if s.starts_with('<') && s.ends_with('>') {
                    Some(s[1..s.len() - 1].to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !placeholders.is_empty() {
            println!(
                "{}",
                format!("Command has {} arguments, please provide their values:", placeholders.len()).dimmed()
            );
            println!();

            for placeholder in placeholders {
                let prompt = Text::new(&format!("{} {}", "Enter the value for the argument".dimmed(), placeholder.bold())).prompt();
                match prompt {
                    Ok(value) => {
                        replaced_command = replaced_command.replace(&format!("<{}>", placeholder), &value);
                    }
                    Err(e) => println!("{}", e.to_string().red()),
                }
            }

            println!("{} {}", "Command:".dimmed(), replaced_command.blue().bold());
        }

        replaced_command
    }
}

#[cfg(test)]
#[cfg(feature = "ollama")]
mod tests {
    use crate::llm::ollama::{config::OllamaConfig, models::OllamaGenerateResponse, ollama_llm::OllamaLocalLlm};

    use super::*;
    use httpmock::{Method::POST, MockServer};

    #[test]
    fn test_simple_suggestion() {
        let mock_generation_response = OllamaGenerateResponse {
            model: "mockstral:latest".to_string(),
            created_at: "2024-06-25T01:40:42.192756+00:00".to_string(),
            response: serde_json::to_string(&SuggestedCommand {
                command: "Mock command".to_string(),
                explanation: "Mock command explanation".to_string(),
            })
            .unwrap(),
            total_duration: 12345,
        };

        let mock_server = MockServer::start();
        let mock_generation_api = mock_server.mock(|when, then| {
            when.method(POST).path("/api/generate");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_generation_response).unwrap());
        });

        let ollama_config = OllamaConfig {
            base_url: Some(mock_server.base_url()),
            model: Some("mockstral:latest".to_string()),
            embedding_model: Some("mockembed:latest".to_string()),
        };

        let ollama = OllamaLocalLlm::new(ollama_config.clone());
        let suggestion_engine = SuggestionEngine::new(Box::new(ollama));

        let suggested_command = suggestion_engine.generate_suggested_command("Mock prompt");
        mock_generation_api.assert();
        assert!(suggested_command.is_ok());
        let suggested_command = suggested_command.unwrap();
        assert!(suggested_command.command.contains("Mock command"));
        assert!(suggested_command.explanation.contains("Mock command explanation"));
    }
}
