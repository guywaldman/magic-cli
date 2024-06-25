use colored::Colorize;
use home::home_dir;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Failed to add command to shell history. Error: {0}")]
    FailedToExecuteCommand(String),

    #[error("Failed to add command to shell history")]
    FailedToAddCommandToHistory,

    #[error("Failed to read shell history")]
    FailedToReadShellHistory,

    #[error("Unknown shell type")]
    UnknownShellType,

    #[error("Unsupported shell type: {0}")]
    UnsupportedShellType(String),
}

#[derive(Debug, PartialEq, Eq)]
enum ShellType {
    Zsh,
    Bash,
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Bash => write!(f, "bash"),
        }
    }
}

impl ShellType {
    fn command_name(&self) -> &str {
        match self {
            ShellType::Zsh => "zsh",
            ShellType::Bash => "bash",
        }
    }

    fn history_file_path(&self) -> PathBuf {
        match self {
            ShellType::Zsh | ShellType::Bash => {
                let home_dir = home_dir().unwrap();
                let history_file_name = match self {
                    ShellType::Zsh => ".zsh_history",
                    ShellType::Bash => ".bash_history",
                };
                home_dir.join(history_file_name)
            }
        }
    }
}

#[derive(Debug)]
pub struct Shell;

impl Shell {
    pub fn add_command_to_history(command: &str) -> Result<(), ShellError> {
        let shell_type = Self::current_shell_type()?;
        let resp = match shell_type {
            ShellType::Zsh | ShellType::Bash => {
                let mut child = Command::new(shell_type.command_name())
                    .arg("-c")
                    .arg(format!(
                        "echo \"{}\" >> {}",
                        command,
                        shell_type.history_file_path().to_str().unwrap()
                    ))
                    .spawn()
                    .map_err(|e| ShellError::FailedToExecuteCommand(e.to_string()))?;
                child.wait().map_err(|e| ShellError::FailedToExecuteCommand(e.to_string()))?
            }
        };

        if !resp.success() {
            println!(
                "{}",
                format!("Failed to add command to shell history. Error: {}", resp.code().unwrap())
                    .red()
                    .bold()
            );
            return Err(ShellError::FailedToAddCommandToHistory);
        } else {
            println!("{}", "Command added to shell history".green().bold());
        }

        Ok(())
    }

    pub(crate) fn get_shell_history() -> Result<Vec<String>, ShellError> {
        let shell_type = Self::current_shell_type()?;
        let resp = match shell_type {
            ShellType::Zsh | ShellType::Bash => {
                let history_file_path = shell_type.history_file_path();
                let history_file = File::open(history_file_path).map_err(|_e| ShellError::FailedToReadShellHistory)?;
                let reader = BufReader::new(history_file);
                reader
                    .lines()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_e| ShellError::FailedToReadShellHistory)?
            }
        };
        Ok(resp)
    }

    fn current_shell_type() -> Result<ShellType, ShellError> {
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                Ok(ShellType::Zsh)
            } else if shell.contains("bash") {
                Ok(ShellType::Bash)
            } else {
                Err(ShellError::UnsupportedShellType(shell))
            }
        } else {
            Err(ShellError::UnknownShellType)
        }
    }
}
