use std::{error::Error, process::exit};

use clap::{Parser, Subcommand};
use colored::Colorize;

use super::{config::CliConfig, explain_subcommand::ExplainSubcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClapCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Explain {
        /// The prompt to explain (e.g., "List all Kubernetes pods")
        #[arg()]
        prompt: String,
    },
    Config {
        #[command(subcommand)]
        command: ConfigSubcommands,
    },
}

#[derive(Subcommand)]
enum ConfigSubcommands {
    /// Set a value.
    Set {
        /// The key to set.
        #[arg(short, long)]
        key: String,
        /// The value to set.
        #[arg(short, long)]
        value: String,
    },
    /// Get a value.
    Get {
        /// The key to get.
        #[arg()]
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
    pub fn run(&self, args: std::env::Args) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        match clap_cli.command {
            Commands::Explain { prompt } => {
                let explain_subcommand = ExplainSubcommand::new(CliConfig::load_config()?);
                explain_subcommand.explain_subcommand(&prompt)?;
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
                    let config = CliConfig::get_config_path()?;
                    println!("{}", config.display());
                }
            },
        }

        Ok(())
    }
}
