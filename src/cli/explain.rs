use std::{
    error::Error,
    process::{exit, Command},
};

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use inquire::{Confirm, Text};

use crate::{engine::Engine, ollama::client::OllamaApiClient};

use super::{
    config::{CliConfig, CommandSuggestMode},
    shell::Shell,
};

#[derive(Debug)]
pub struct ExplainSubcommand {
    ollama_client: OllamaApiClient,
    config: CliConfig,
}

impl ExplainSubcommand {
    pub fn new(config: CliConfig) -> Self {
        let ollama_client = OllamaApiClient::new(config.ollama_config.clone());
        Self { ollama_client, config }
    }

    pub fn preamble(&self) -> Result<(), Box<dyn Error>> {
        let running_models = self.ollama_client.list_running_models()?;
        if running_models.models.is_empty() {
            println!(
                "{}",
                "No running Ollama models, response may take longer than usual to generate.".yellow()
            );
        }
        Ok(())
    }

    pub fn explain_subcommand(&self, prompt: &str) -> Result<(), Box<dyn Error>> {
        self.preamble()?;

        let engine = Engine::new(self.ollama_client.clone());
        println!(
            "{} {}",
            "Generating suggested command for prompt".dimmed(),
            format!("\"{}\"...", prompt).dimmed(),
        );
        println!();

        let suggested_command = engine.suggest_command(prompt)?;

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
                    let revision_respose = engine.suggest_command_with_revision(&suggested_command, &revision_prompt)?;
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

        if let Some(exit_code) = self.finalize_command_suggestion(&command)? {
            exit(exit_code);
        }

        Ok(())
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
