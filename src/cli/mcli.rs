use super::{
    config::MagicCliConfig,
    subcommand::{MagicCliRunOptions, MagicCliSubcommand},
    subcommand_ask::AskSubcommand,
    subcommand_config::{ConfigSubcommand, ConfigSubcommands},
    subcommand_search::SearchSubcommand,
    subcommand_suggest::{SuggestSubcommand, SuggestSubcommandArguments},
    subcommand_sysinfo::SysInfoSubcommand,
};
use clap::{ArgAction, Parser, Subcommand};
use colored::Colorize;
use std::{error::Error, path::PathBuf, process::exit};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct ClapCli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    /// Path to a custom configuration file (JSON).
    /// By default, the configuration file is located at ~/.config/magic_cli/config.json.
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Suggest a command.
    Suggest {
        /// The prompt to suggest a command for (e.g., "List all Kubernetes pods")
        #[arg()]
        prompt: String,

        #[arg(short, long, action=ArgAction::SetTrue)]
        output_only: bool,
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
    pub async fn run(&self, args: &[String]) -> Result<(), Box<dyn Error>> {
        let clap_cli = ClapCli::parse_from(args);

        let ClapCli {
            command: subcommand,
            config,
        } = clap_cli;

        let config = MagicCliConfig::new(config);

        match subcommand {
            Commands::Suggest { prompt, output_only, .. } => {
                Self::run_subcommmand(&config, SuggestSubcommand::new(SuggestSubcommandArguments { prompt, output_only })).await;
            }
            Commands::Ask { prompt, .. } => {
                Self::run_subcommmand(&config, AskSubcommand::new(prompt)).await;
            }
            Commands::Config { command, .. } => {
                Self::run_subcommmand(&config, ConfigSubcommand::new(command)).await;
            }
            Commands::Search { prompt, index, .. } => {
                Self::run_subcommmand(&config, SearchSubcommand::new(prompt, index)).await;
            }
            Commands::SysInfo => {
                Self::run_subcommmand(&config, SysInfoSubcommand::new()).await;
            }
        }

        Ok(())
    }

    async fn run_subcommmand(config: &MagicCliConfig, subcommand: impl MagicCliSubcommand) {
        let run_options = MagicCliRunOptions { config: config.clone() };
        match subcommand.run(run_options).await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Error: {}", err).red().bold());
                exit(1);
            }
        }
    }
}
