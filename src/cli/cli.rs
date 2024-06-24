use std::error::Error;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use inquire::{Confirm, Text};

use crate::{
    engine::{Engine, SuggestedCommand},
    ollama::{client::OllamaApiClient, config::OllamaConfig},
};

use clap::{Args, Parser, Subcommand};

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
pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, args: std::env::Args) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        match clap_cli.command {
            Commands::Explain(explain_subcommand) => {
                let prompt = explain_subcommand.prompt;
                self.explain_subcommand(&prompt).unwrap();
            }
        }

        Ok(())
    }

    fn explain_subcommand(&self, prompt: &str) -> Result<(), Box<dyn Error>> {
        let engine = Engine::new(OllamaApiClient::new(OllamaConfig::default()));
        println!(
            "{} {}",
            "Generating suggested command for prompt".dimmed(),
            format!("\"{}\"...", prompt.italic()).dimmed(),
        );
        println!();

        let suggested_command = engine.suggest_command(prompt)?;

        println!("{:>18}: {}", "Suggested command".dimmed(), suggested_command.command.blue().bold());
        println!("{:>18}: {}", "Explanation".dimmed(), suggested_command.explanation.italic());
        println!();

        let confirm = Confirm::new("Copy to clipboard?").with_default(true).prompt();
        match confirm {
            Ok(true) => {
                let mut clipboard_context = ClipboardContext::new()?;
                clipboard_context.set_contents(suggested_command.command)?;
                println!("{}", "Command copied to clipboard".green().bold());
            }
            Ok(false) => println!("{}", "Suggested command not copied to clipboard".red()),
            Err(e) => println!("{}", e.to_string().red()),
        }
        // dbg!(&suggested_command);
        Ok(())
    }
}
