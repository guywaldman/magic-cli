use super::{command::CliCommand, config::MagicCliConfig, subcommand::MagicCliSubcommand};
use crate::{
    cli::command::CommandRunResult,
    core::{AskEngine, AskResponse},
};
use colored::Colorize;
use std::error::Error;

pub struct AskSubcommand {
    prompt: String,
}

impl AskSubcommand {
    pub fn new(prompt: String) -> Self {
        Self { prompt }
    }

    fn create_context(history: &[String]) -> String {
        history.iter().map(|item| format!("- {}", item)).collect::<Vec<_>>().join("\n")
    }

    fn print_response(response: &AskResponse) {
        match response {
            AskResponse::Suggest(suggest_response) => {
                println!("{}", "Suggestion:".green().bold());
                println!("{}", format!("  - Command: {}", suggest_response.command.blue().bold()));
                println!("{}", format!("  - Explanation: {}", suggest_response.explanation.italic()));
            }
            AskResponse::Ask(ask_response) => {
                println!("{}", "Action required:".yellow().bold());
                println!("{}", format!("  - Command: {}", ask_response.command.blue().bold()));
                println!("{}", format!("  - Rationale: {}", ask_response.rationale.italic()));
            }
            AskResponse::Success(success_response) => {
                println!("{}", "Success:".green().bold());
                println!(
                    "{}",
                    format!("  - Success: {}", success_response.success.to_string().green().bold())
                );
            }
        }
    }
}

impl MagicCliSubcommand for AskSubcommand {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let config = MagicCliConfig::load_config()?;
        let llm = MagicCliConfig::llm_from_config(&config)?;
        println!("{}", "Model details:".dimmed());
        println!("{}", format!("  - LLM provider: {}", &llm.provider()).dimmed());
        println!(
            "{}",
            format!("  - Text completion model: {}", &llm.text_completion_model_name()).dimmed()
        );
        println!("{}", format!("  - Embedding model: {}", &llm.embedding_model_name()).dimmed());

        println!("\nGenerating initial response from model...");

        let mut history: Vec<String> = vec![format!("User has requested the ask '{}'.", self.prompt)];
        let ask_engine = AskEngine::new(llm);
        let mut command = ask_engine.ask_command(&self.prompt)?;
        loop {
            Self::print_response(&command);
            match command {
                AskResponse::Success(_) => {
                    println!("{}", "Successfully completed the ask".green().bold());
                    break;
                }
                AskResponse::Suggest(ref suggest_response) => {
                    let command_run_result =
                        CliCommand::new(config.suggest.clone()).suggest_user_action_on_command(&suggest_response.command)?;
                    if let CommandRunResult::Execution(execution_result) = command_run_result {
                        history.push(format!(
                            "User has ran the command '{}' with a status code of {} and stdout of '{}' and stderr of '{}'.",
                            &suggest_response.command, execution_result.status, execution_result.stdout, execution_result.stderr
                        ))
                    }
                    command = ask_engine.ask_command_with_context(&suggest_response.command, &Self::create_context(&history))?;
                }
                AskResponse::Ask(ask_response) => {
                    let command_run_result =
                        CliCommand::new(config.suggest.clone()).suggest_user_action_on_command(&ask_response.command)?;

                    println!(
                        "{}",
                        format!("ACTION REQUIRED: Run {}", format!("`{}`", ask_response.command.blue().bold())).yellow()
                    );
                    println!("{}", format!("RATIONALE: {}", ask_response.rationale).dimmed().italic());

                    if let CommandRunResult::Execution(execution_result) = command_run_result {
                        history.push(format!(
                            "User has ran the command '{}' with a status code of {} and stdout of '{}' and stderr of '{}'.",
                            ask_response.command, execution_result.status, execution_result.stdout, execution_result.stderr
                        ));
                    }
                    command = ask_engine.ask_command_with_context(&ask_response.command, &Self::create_context(&history))?;

                    continue;
                }
            }
        }
        println!("{}", "Successfully completed the ask".green().bold());
        Ok(())
    }
}
