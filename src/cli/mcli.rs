use super::{
    command::CliCommand,
    config::{self, CliConfig, LlmProvider},
    search::CliSearch,
};
use crate::{
    core::{Llm, SuggestionEngine},
    llm::{ollama::ollama_llm::OllamaLocalLlm, openai::openai_llm::OpenAiLlm},
};
use clap::{ArgAction, Parser, Subcommand};
use colored::Colorize;
use inquire::Text;
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

#[derive(Subcommand)]
enum ConfigSubcommands {
    /// Set a value.
    Set {
        /// The key to set in the configuration. If not provided, you will be prompted to select one.
        #[arg(short, long)]
        key: Option<String>,
        /// The value to set. If not provided, you will be prompted to enter a value.
        #[arg(short, long)]
        value: Option<String>,
    },
    /// Get a value.
    Get {
        /// The key to get from the configuration. If not provided, you will be prompted to select one.
        #[arg()]
        key: Option<String>,
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
                let llm = self.llm(&config)?;
                let explain_subcommand = SuggestionEngine::new(llm);
                let command = explain_subcommand.suggest_command(&prompt)?;
                if CliCommand::new(config.suggest).suggest_user_action_on_command(&command).is_err() {
                    exit(1);
                }
            }
            Commands::Config { command } => match command {
                ConfigSubcommands::Set { key, value } => {
                    let key = match key {
                        Some(key) => key,
                        None => config::CliConfig::select_key()?,
                    };
                    let value = match value {
                        Some(value) => value,
                        // TODO: Support secrets.
                        None => Text::new(&format!("{} {}: ", "Enter the value for the key", key.magenta())).prompt()?,
                    };

                    match CliConfig::set(&key, &value) {
                        Ok(_) => println!("{}", "Configuration updated.".green().bold()),
                        Err(err) => {
                            eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                            exit(1);
                        }
                    }
                }
                ConfigSubcommands::Get { key } => {
                    let key = match key {
                        Some(key) => key,
                        None => config::CliConfig::select_key()?,
                    };
                    match CliConfig::get(&key) {
                        Ok(value) => println!("{}", value),
                        Err(err) => {
                            eprintln!("{}", format!("CLI configuration error: {}", err).red().bold());
                            exit(1);
                        }
                    }
                }
                ConfigSubcommands::List => {
                    let config_keys = CliConfig::configuration_keys();
                    let config_keys = config_keys.get().unwrap();
                    let mut config_keys_sorted = config_keys.values().collect::<Vec<_>>();
                    config_keys_sorted.sort_by(|a, b| a.key.cmp(&b.key));
                    for (i, item) in config_keys_sorted.iter().enumerate() {
                        let config_value = CliConfig::get(&item.key)?;
                        let config_value = config_value.replace("null", "-");
                        let config_value = if item.is_secret {
                            "*".repeat(config_value.len())
                        } else {
                            config_value
                        };
                        println!(
                            "Field: {} {}\nValue: {}\nDescription: {}",
                            item.key.blue().bold(),
                            if item.is_secret { "(secret)".yellow() } else { "".dimmed() },
                            config_value.bold(),
                            item.description.dimmed(),
                        );
                        if i < config_keys_sorted.len() - 1 {
                            println!();
                        }
                    }
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
                let config = CliConfig::load_config()?;
                let llm = self.llm(&config)?;
                let cli_search = CliSearch::new(llm);
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

    fn llm(&self, config: &CliConfig) -> Result<Box<dyn Llm>, Box<dyn Error>> {
        match config.llm {
            LlmProvider::Ollama => Ok(Box::new(OllamaLocalLlm::new(config.ollama_config.clone()))),
            LlmProvider::OpenAi => Ok(Box::new(OpenAiLlm::new(config.openai_config.clone()))),
        }
    }
}
