use super::{
    subcommand::MagicCliSubcommand,
    subcommand_ask::AskSubcommand,
    subcommand_config::{ConfigSubcommand, ConfigSubcommands},
    subcommand_search::SearchSubcommand,
    subcommand_suggest::SuggestSubcommand,
    subcommand_sysinfo::SysInfoSubcommand,
};
use clap::{ArgAction, Parser, Subcommand};
use colored::Colorize;
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
    /// Ask to perform an action in the terminal.
    Ask {
        /// The prompt to ask for (e.g., "Set up the development environment")
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
                Self::run_subcommmand(SuggestSubcommand::new(prompt));
            }
            Commands::Ask { prompt } => {
                Self::run_subcommmand(AskSubcommand::new(prompt));
            }
            Commands::Config { command } => {
                Self::run_subcommmand(ConfigSubcommand::new(command));
            }
            Commands::Search { prompt, index } => {
                Self::run_subcommmand(SearchSubcommand::new(prompt, index));
            }
            Commands::SysInfo => {
                Self::run_subcommmand(SysInfoSubcommand::new());
            }
        }

        Ok(())
    }

    fn run_subcommmand(subcommand: impl MagicCliSubcommand) {
        match subcommand.run() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Error: {}", err).red().bold());
                exit(1);
            }
        }
    }
}
