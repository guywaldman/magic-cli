use colored::Colorize;
use inquire::Text;
use orch::{
    execution::{StructuredExecutorBuilder, TextExecutorBuilder},
    lm::LanguageModel,
    response::{variants, Variant, Variants},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::Shell;

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

#[derive(Variants, serde::Deserialize)]
pub enum SuggestResponseVariant {
    Command(SuggestResponseCommand),
    Error(SuggestResponseError),
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Command",
    scenario = "You are confident enough in the command you want to suggest",
    description = "Suggestion for a command to run and its explanation. The command should be simple, adhere to the user's request and system information, and should only contain placeholders if deemed necessary."
)]
pub struct SuggestResponseCommand {
    #[schema(description = "The command to run", example = "kubectl get pods")]
    pub command: String,
    #[schema(description = "The explanation for the command", example = "List all Kubernetes pods")]
    pub explanation: String,
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Error",
    scenario = "You are not confident enough in the command you want to suggest",
    description = "Error message for the user"
)]
pub struct SuggestResponseError {
    #[schema(
        description = "The error message, should be descriptive",
        example = "'I could not understand the instructions' or 'I could not find a suitable command'"
    )]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestConfig {
    pub mode: Option<SuggestMode>,
    pub add_to_history: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedCommand {
    pub command: String,
    pub explanation: String,
}

#[derive(Error, Debug)]
pub enum SuggestionEngineError {
    #[error("{0}")]
    Configuration(String),

    #[error("{0}")]
    Generation(String),

    #[error("{0}")]
    Serialization(String),
}

pub struct SuggestionEngine<'a> {
    lm: &'a dyn LanguageModel,
}

impl<'a> SuggestionEngine<'a> {
    pub fn new(lm: &'a dyn LanguageModel) -> Self {
        Self { lm }
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
    pub async fn suggest_command(&self, prompt: &str, output_only: bool) -> Result<String, SuggestionEngineError> {
        if !output_only {
            println!(
                "{} {}",
                "Generating suggested command for prompt".dimmed(),
                format!("\"{}\"...", prompt).dimmed(),
            );
            println!();
        }

        let suggested_command = self
            .generate_suggested_command(prompt)
            .await
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;

        if !output_only {
            println!("{:>18}: {}", "Suggested command".dimmed(), suggested_command.command.blue().bold());
            println!("{:>18}: {}", "Explanation".dimmed(), suggested_command.explanation.italic());
            println!();
        }

        let mut command = suggested_command.command.clone();

        if !output_only {
            // In "output only" mode, skip the interactive prompts.
            loop {
                let revise_command = Text::new("Provide instructions on how to revise the command (leave empty to skip)").prompt();
                match revise_command {
                    Ok(revision_prompt) => {
                        if revision_prompt.trim().is_empty() {
                            break;
                        }
                        let revision_respose = self
                            .generate_suggested_command_with_revision(&suggested_command, &revision_prompt)
                            .await?;
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
            command = self.populate_placeholders_in_command(&command);
        }

        if !output_only {
            println!();
        }

        Ok(command)
    }

    pub(crate) async fn generate_suggested_command(&self, prompt: &str) -> Result<SuggestedCommand, SuggestionEngineError> {
        let system_info = Shell::extract_env_info().unwrap();

        let preamble = format!(
            "
        You are a an assistant that provides suggestions for a command to run on the user's command line.

        The user's system information is:
        - Shell: {shell}
        - OS: {os}
        - OS version: {os_version}
        - CPU architecture: {arch}
        ",
            shell = system_info.shell,
            os = system_info.os,
            os_version = system_info.os_version,
            arch = system_info.arch
        );

        let executor = StructuredExecutorBuilder::new()
            .with_lm(self.lm)
            .with_preamble(&preamble)
            .with_options(Box::new(variants!(SuggestResponseVariant)))
            .try_build()
            .map_err(|e| SuggestionEngineError::Configuration(e.to_string()))?;
        let response = executor
            .execute(prompt)
            .await
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?
            .content;

        match response {
            SuggestResponseVariant::Command(command) => Ok(SuggestedCommand {
                command: command.command,
                explanation: command.explanation,
            }),
            SuggestResponseVariant::Error(error) => Err(SuggestionEngineError::Generation(error.error)),
        }
    }

    pub(crate) async fn generate_suggested_command_with_revision(
        &self,
        previous_command: &SuggestedCommand,
        prompt: &str,
    ) -> Result<SuggestedCommand, SuggestionEngineError> {
        let system_info = Shell::extract_env_info().unwrap();

        let system_prompt = format!("
        You are a an assistant that provides suggestions for a command to run on the user's command line.
        Please only provide a JSON which contains the fields:
        - 'command' for the command,
        - 'explanation' for a very short explanation about the command or notes you may have
        ...and nothing else! Just those fields.
        For the command, if there are arguments, use the format '<argument_name>', for example 'kubectl logs -n <namespace> <pod-name>'.

        You will receive a command that you suggested before and a prompt from the user to revise it.

        IMPORTANT NOTES:
        1. Remember to format the response as JSON.
        2. Make sure that *any* placeholder (even things such as /path/to/file) are formatted as '<placeholder_name>'.
        3. Try to accommodate for the user's environment (adjust the suggestion based on their shell, OS, architecture, etc.). See below the system information.

        The user's system information is:
        - Shell: {shell}
        - OS: {os}
        - OS version: {os_version}
        - CPU architecture: {arch}
        ", shell = system_info.shell, os = system_info.os, os_version = system_info.os_version, arch = system_info.arch);

        let prompt = format!(
            "
            The previous command was: `{}` (explanation: '{}')
            Please revise the command to satisfy the user's request.
            The revision prompt is: '{}'
        ",
            previous_command.command, previous_command.explanation, prompt
        );

        let executor = TextExecutorBuilder::new()
            .with_lm(self.lm)
            .with_preamble(&system_prompt)
            .try_build()
            .map_err(|e| SuggestionEngineError::Configuration(e.to_string()))?;
        let response = executor
            .execute(&prompt)
            .await
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?
            .content;
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

mod tests {
    use super::*;
    use httpmock::{Method::POST, MockServer};
    use orch::lm::{LanguageModelBuilder, OllamaBuilder, OllamaGenerateResponse};

    #[tokio::test]
    async fn test_simple_suggestion() {
        let mock_generation_response = OllamaGenerateResponse {
            model: "mockstral:latest".to_string(),
            created_at: "2024-06-25T01:40:42.192756+00:00".to_string(),
            response: serde_json::to_string(&SuggestedCommand {
                command: "Mock command".to_string(),
                explanation: "Mock command explanation".to_string(),
            })
            .unwrap(),
            total_duration: 12345,
            context: None,
        };

        let mock_server = MockServer::start();
        let mock_generation_api = mock_server.mock(|when, then| {
            when.method(POST).path("/api/generate");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_generation_response).unwrap());
        });

        let base_url = mock_server.base_url();
        let ollama = OllamaBuilder::new()
            .with_base_url(base_url)
            .with_model("mockstral:latest".to_string())
            .with_embeddings_model("mockembed:latest".to_string())
            .try_build()
            .unwrap();
        let suggestion_engine = SuggestionEngine::new(&ollama);

        let suggested_command = suggestion_engine.generate_suggested_command("Mock prompt").await;
        mock_generation_api.assert();
        assert!(suggested_command.is_ok());
        let suggested_command = suggested_command.unwrap();
        assert!(suggested_command.command.contains("Mock command"));
        assert!(suggested_command.explanation.contains("Mock command explanation"));
    }
}
