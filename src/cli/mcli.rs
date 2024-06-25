use super::{command::CliCommand, config::CliConfig, search::CliSearch};
use crate::{core::SuggestionEngine, ollama::ollama_llm::OllamaLocalLlm};
use clap::{ArgAction, Parser, Subcommand};
use colored::Colorize;
use const_format::concatcp;
use std::{error::Error, process::exit};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClapCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Suggest {
        /// The prompt to suggest a command for (e.g., "List all Kubernetes pods")
        #[arg()]
        prompt: String,
    },
    Config {
        #[command(subcommand)]
        command: ConfigSubcommands,
    },
    Search {
        /// The prompt to search for.
        #[arg()]
        prompt: String,

        /// Whether to index previously executed commands (this may take a while).
        #[arg(short, long, action=ArgAction::SetTrue)]
        index: bool,
    },
}

const KEY_HELP: &str = "Supported keys:
    - suggest.mode: The mode to use for suggesting commands (supported values: \"clipboard\" for copying to clipboard, \"unsafe-execution\" for executing commands in the current shell session).
    - suggest.add_to_history: Whether to add the suggested command to the shell history (supported values: \"true\", \"false\").
    - ollama.base_url: The base URL of the Ollama API.
    - ollama.model: The model to use for generating responses.
    - ollama.embedding_model: The model to use for generating embeddings.";

const SET_KEY_HELP: &str = concatcp!("Sets a key in the configuration.\n", KEY_HELP);
const GET_KEY_HELP: &str = concatcp!("Gets a key from the configuration.\n", KEY_HELP);

#[derive(Subcommand)]
enum ConfigSubcommands {
    /// Set a value.
    Set {
        /// The key to set in the configuration.
        #[arg(short, long, long_help=SET_KEY_HELP)]
        key: String,
        /// The value to set.
        #[arg(short, long)]
        value: String,
    },
    /// Get a value.
    Get {
        /// The key to get from the configuration.
        #[arg(long_help=GET_KEY_HELP)]
        key: String,
    },
    /// List the configurations.
    List,
    /// Reset the configurations to the default values.
    Reset,
    /// Get the path to the configuration file.
    Path,
}

pub struct MagicCli;

impl MagicCli {
    pub fn run(&self, args: &[String]) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        match clap_cli.command {
            Commands::Suggest { prompt } => {
                let config = CliConfig::load_config()?;
                let llm = OllamaLocalLlm::new(config.ollama_config.clone());
                let explain_subcommand = SuggestionEngine::new(llm);
                let command = explain_subcommand.suggest_command(&prompt)?;
                if CliCommand::new(config.suggest).suggest_user_action_on_command(&command).is_err() {
                    exit(1);
                }
            }
            Commands::Config { command } => match command {
                ConfigSubcommands::Set { key, value } => match CliConfig::set(&key, &value) {
                    Ok(_) => println!("{}", "Configuration updated.".green().bold()),
                    Err(err) => {
                        eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                        exit(1);
                    }
                },
                ConfigSubcommands::Get { key } => match CliConfig::get(&key) {
                    Ok(value) => println!("{}", value),
                    Err(err) => {
                        eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                        exit(1);
                    }
                },
                ConfigSubcommands::List => {
                    let config = CliConfig::load_config()?;
                    println!("{}", config);
                }
                ConfigSubcommands::Reset => {
                    CliConfig::reset()?;
                    println!("{}", "Configuration reset to default values.".green().bold());
                }
                ConfigSubcommands::Path => {
                    let config = CliConfig::get_config_file_path()?;
                    println!("{}", config.display());
                }
            },
            Commands::Search { prompt, index } => {
                let cli_search = CliSearch;
                let selected_command = cli_search.search_command(&prompt, index)?;

                let config = CliConfig::load_config()?;
                if CliCommand::new(config.suggest)
                    .suggest_user_action_on_command(&selected_command)
                    .is_err()
                {
                    exit(1);
                }
            }
        }

        Ok(())
    }
}
