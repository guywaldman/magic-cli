use std::error::Error;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use inquire::{Confirm, Text};
use thiserror::Error;

use crate::{
    engine::Engine,
    ollama::{client::OllamaApiClient, config::OllamaConfig},
};

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub ollama_config: OllamaConfig,
}

#[derive(Error, Debug)]
pub enum CliConfigError {
    // #[error("Configuration file not found: {0}")]
    // MissingConfigFile(String),
}

impl CliConfig {
    pub fn load_config() -> Result<Self, CliConfigError> {
        // TODO: Load config from file.
        Ok(Self {
            ollama_config: Default::default(),
        })
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClapCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Explain(ExplainSubcommand),
}

#[derive(Debug, Args)]
struct ExplainSubcommand {
    /// The prompt to explain (e.g., "List all Kubernetes pods")
    #[arg()]
    prompt: String,
}

#[derive(Debug)]
pub struct Cli {
    ollama_client: OllamaApiClient,
}

impl Cli {
    pub fn new(config: CliConfig) -> Self {
        let ollama_client = OllamaApiClient::new(config.ollama_config);
        Self { ollama_client }
    }

    pub fn run(&self, args: std::env::Args) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        match clap_cli.command {
            Commands::Explain(explain_subcommand) => {
                self.preamble()?;
                let prompt = explain_subcommand.prompt;
                self.explain_subcommand(&prompt).unwrap();
            }
        }

        Ok(())
    }

    fn preamble(&self) -> Result<(), Box<dyn Error>> {
        let running_models = self.ollama_client.list_running_models()?;
        if running_models.models.is_empty() {
            println!(
                "{}",
                "No running Ollama models, response may take longer than usual to generate.".yellow()
            );
        }
        Ok(())
    }

    fn explain_subcommand(&self, prompt: &str) -> Result<(), Box<dyn Error>> {
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

        let confirm = Confirm::new("Copy to clipboard?").with_default(true).prompt();
        match confirm {
            Ok(true) => {
                let mut clipboard_context = ClipboardContext::new()?;
                clipboard_context.set_contents(command)?;
                println!("{}", "Command copied to clipboard".green().bold());
            }
            Ok(false) => println!("{}", "Suggested command not copied to clipboard".red()),
            Err(e) => println!("{}", e.to_string().red()),
        }
        Ok(())
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
