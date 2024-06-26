use super::{
    subcommand_config::{ConfigSubcommand, ConfigSubcommands},
    subcommand_search::SearchSubcommand,
    subcommand_suggest::SuggestSubcommand,
    subcommand_sysinfo::SysInfoSubcommand,
};
use clap::{ArgAction, Parser, Subcommand};
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
    /// Suggest a command.
    Suggest {
        /// The prompt to suggest a command for (e.g., "List all Kubernetes pods")
        #[arg()]
        prompt: String,
    },
    /// Configure the CLI.
    Config {
        #[command(subcommand)]
        command: ConfigSubcommands,
    },
    /// Search the command history
    Search {
        /// The prompt to search for.
        #[arg()]
        prompt: String,

        /// Whether to index previously executed commands (this may take a while).
        #[arg(short, long, action=ArgAction::SetTrue)]
        index: bool,
    },
    /// Get system information.
    SysInfo,
}

pub struct MagicCli;

impl MagicCli {
    pub fn run(&self, args: &[String]) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        match clap_cli.command {
            Commands::Suggest { prompt } => {
                if SuggestSubcommand::run(&prompt).is_err() {
                    exit(1);
                }
            }
            Commands::Config { command } => {
                if ConfigSubcommand::run(&command).is_err() {
                    exit(1);
                }
            }
            Commands::Search { prompt, index } => {
                if SearchSubcommand::run(&prompt, index).is_err() {
                    exit(1);
                }
            }
            Commands::SysInfo => {
                if SysInfoSubcommand::run().is_err() {
                    exit(1);
                }
            }
        }

        Ok(())
    }
}
