use std::process::Command;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;
use inquire::Confirm;
use thiserror::Error;

use crate::core::{Shell, ShellError, SuggestConfig, SuggestMode};

#[derive(Debug, Error)]
pub(crate) enum CliCommandError {
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Failed to interact with shell: {0}")]
    Shell(#[from] ShellError),
}

pub(crate) struct CliCommand {
    suggest_config: SuggestConfig,
}

impl CliCommand {
    pub fn new(suggest_config: SuggestConfig) -> Self {
        Self { suggest_config }
    }
    pub fn suggest_user_action_on_command(&self, command: &str) -> Result<(), CliCommandError> {
        match self.suggest_config.mode {
            SuggestMode::Clipboard => {
                let confirm = Confirm::new("Copy to clipboard?").with_default(true).prompt();
                match confirm {
                    Ok(true) => {
                        let mut clipboard_context = ClipboardContext::new().map_err(|e| CliCommandError::Clipboard(e.to_string()))?;
                        clipboard_context
                            .set_contents(command.to_string())
                            .map_err(|e| CliCommandError::Clipboard(e.to_string()))?;
                        println!("{}", "Command copied to clipboard".green().bold());
                    }
                    Ok(false) => println!("{}", "Suggested command not copied to clipboard".red()),
                    Err(e) => println!("{}", e.to_string().red()),
                }
            }
            SuggestMode::Execution => {
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
                    Ok(false) => return Ok(()),
                    Ok(true) => {}
                    Err(e) => println!("{}", e.to_string().red()),
                }

                // TODO: Handle error.
                let system_info = Shell::extract_env_info().unwrap();
                let child = Command::new(system_info.shell)
                    .arg("-c")
                    .arg(command)
                    .current_dir(std::env::current_dir().unwrap())
                    .spawn()
                    .expect("Failed to execute command");

                let output = child.wait_with_output().expect("Failed to wait for command");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let status = output.status;

                if self.suggest_config.add_to_history {
                    Shell::add_command_to_history(command)?;
                }

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
                    return Ok(());
                }

                if !stdout.is_empty() {
                    println!("stdout:\n{}\n", stderr);
                }
                if !stderr.is_empty() {
                    println!("stderr:\n{}\n", stdout);
                }
            }
        }
        Ok(())
    }
}
