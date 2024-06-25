use std::{
    error::Error,
    process::{exit, Command},
};

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ollama::{client::OllamaApiClient, models::OllamaApiModelsMetadata};

use super::{
    config::{CliConfig, CommandSuggestMode},
    shell::Shell,
};

#[derive(Debug)]
pub struct SuggestionEngine {
    ollama_client: OllamaApiClient,
    config: CliConfig,
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

    #[error("Model listing error: {0}")]
    ModelListing(String),

    #[error("Command finalization error: {0}")]
    CommandFinalization(String),

    #[error("Serialization or deserialization error: {0}")]
    Serialization(String),
}

impl SuggestionEngine {
    pub fn new(config: CliConfig) -> Self {
        let ollama_client = OllamaApiClient::new(config.ollama_config.clone());
        Self { ollama_client, config }
    }

    pub fn preamble(&self) -> Result<(), SuggestionEngineError> {
        let running_models = self.list_running_models()?;
        if running_models.models.is_empty() {
            println!(
                "{}",
                "No running Ollama models, response may take longer than usual to generate.".yellow()
            );
        }
        Ok(())
    }

    pub fn suggest_command(&self, prompt: &str) -> Result<(), SuggestionEngineError> {
        self.preamble()?;

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

        if let Some(exit_code) = self
            .finalize_command_suggestion(&command)
            .map_err(|e| SuggestionEngineError::CommandFinalization(e.to_string()))?
        {
            exit(exit_code);
        }

        Ok(())
    }

    pub(crate) fn list_running_models(&self) -> Result<OllamaApiModelsMetadata, SuggestionEngineError> {
        let response = self
            .ollama_client
            .list_running_models()
            .map_err(|e| SuggestionEngineError::ModelListing(e.to_string()))?;
        Ok(response)
    }

    pub(crate) fn generate_suggested_command(&self, prompt: &str) -> Result<SuggestedCommand, SuggestionEngineError> {
        const SYSTEM_PROMPT: &str = "
        You are a an assistant that provides suggestions for a command to
        run on a Linux machine that satisfies the user's request.
        Please only provide a JSON which contains the fields 'command' for the command,
        'explanation' for a very short explanation about the command or notes you may have,
        and nothing else.
        For the command, if there are arguments, use the format '<argument_name>', for example 'kubectl logs -n <namespace> <pod-name>'.
        ";

        let response = self
            .ollama_client
            .generate(prompt, SYSTEM_PROMPT)
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;
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
            .ollama_client
            .generate(&prompt, SYSTEM_PROMPT)
            .map_err(|e| SuggestionEngineError::Generation(e.to_string()))?;
        let parsed_response = serde_json::from_str(&response).map_err(|e| SuggestionEngineError::Serialization(e.to_string()))?;
        Ok(parsed_response)
    }

    fn finalize_command_suggestion(&self, command: &str) -> Result<Option<i32>, Box<dyn Error>> {
        match self.config.suggest.mode {
            CommandSuggestMode::Clipboard => {
                let confirm = Confirm::new("Copy to clipboard?").with_default(true).prompt();
                match confirm {
                    Ok(true) => {
                        let mut clipboard_context = ClipboardContext::new()?;
                        clipboard_context.set_contents(command.to_string())?;
                        println!("{}", "Command copied to clipboard".green().bold());
                    }
                    Ok(false) => println!("{}", "Suggested command not copied to clipboard".red()),
                    Err(e) => println!("{}", e.to_string().red()),
                }
            }
            CommandSuggestMode::Execution => {
                let confirm = Confirm::new(&format!("Execute command '{}'?", command.blue().bold()))
                    .with_default(true)
                    .with_help_message(&format!(
                        "{}",
                        "WARNING: This will execute the command in the current session, please make sure that it is safe to do so"
                            .red()
                            .bold()
                    ))
                    .prompt();

                match confirm {
                    Ok(false) => return Ok(None),
                    Ok(true) => {}
                    Err(e) => println!("{}", e.to_string().red()),
                }

                let split_command = command.split_whitespace().collect::<Vec<_>>();
                let child = Command::new(split_command[0])
                    .args(&split_command[1..])
                    .spawn()
                    .expect("Failed to execute command");

                let output = child.wait_with_output().expect("Failed to wait for command");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let status = output.status;

                Shell::add_command_to_history(command)?;

                if !status.success() {
                    println!(
                        "{}",
                        format!("Command `{}` failed with status code {}", command.italic(), status.code().unwrap())
                            .red()
                            .bold()
                    );
                    if !stdout.is_empty() {
                        println!("stdout:\n{}\n", stderr);
                    }
                    if !stderr.is_empty() {
                        println!("stderr:\n{}\n", stdout);
                    }
                    return Ok(None);
                }

                if !stdout.is_empty() {
                    println!("stdout:\n{}\n", stderr);
                }
                if !stderr.is_empty() {
                    println!("stderr:\n{}\n", stdout);
                }
            }
        }
        Ok(None)
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
    use crate::ollama::models::{OllamaApiModelDetails, OllamaApiModelMetadata, OllamaApiModelsMetadata, OllamaGenerateResponse};
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };

    #[test]
    fn test_simple_suggestion() {
        let now = chrono::Utc::now().to_rfc3339();
        let mock_generation_response = OllamaGenerateResponse {
            model: "mockstral:latest".to_string(),
            created_at: now.clone(),
            response: serde_json::to_string(&SuggestedCommand {
                command: "Mock command".to_string(),
                explanation: "Mock command explanation".to_string(),
            })
            .unwrap(),
            total_duration: 12345,
        };
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
        let mock_generation_api = mock_server.mock(|when, then| {
            when.method(POST).path("/api/generate");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_generation_response).unwrap());
        });
        let mock_list_models_api = mock_server.mock(|when, then| {
            when.method(GET).path("/api/ps");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&mock_list_models_response).unwrap());
        });

        let mut cli_config = CliConfig::default();
        cli_config.ollama_config.base_url = mock_server.base_url();

        let suggestion_engine = SuggestionEngine::new(cli_config);

        let running_models = suggestion_engine.list_running_models();
        mock_list_models_api.assert();
        assert!(running_models.is_ok());
        let running_models = running_models.unwrap();
        assert!(running_models.models.len() == 1);
        let model = running_models.models.first().unwrap();
        assert_eq!(model.name, mock_list_models_response.models.first().unwrap().name);

        let suggested_command = suggestion_engine.generate_suggested_command("Mock prompt");
        mock_generation_api.assert();
        assert!(suggested_command.is_ok());
        let suggested_command = suggested_command.unwrap();
        assert!(suggested_command.command.contains("Mock command"));
        assert!(suggested_command.explanation.contains("Mock command explanation"));
    }
}
