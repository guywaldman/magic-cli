use colored::Colorize;
use home::home_dir;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub shell: String,
    pub os: String,
    pub os_version: String,
    pub arch: String,
}

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Failed to add command to shell history. Error: {0}")]
    FailedToExecuteCommand(String),

    #[error("Failed to add command to shell history")]
    FailedToAddCommandToHistory,

    #[error("Failed to read shell history")]
    FailedToReadShellHistory,

    #[error("Unsupported shell type: {0}")]
    UnsupportedShellType(String),

    #[error("Failed to extract system info: {0}")]
    FailedToExtractSystemInfo(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellType {
    Zsh,
    Bash,
    Pwsh,
}

impl TryFrom<&str> for ShellType {
    type Error = ShellError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "zsh" => Ok(ShellType::Zsh),
            "bash" => Ok(ShellType::Bash),
            "powershell" => Ok(ShellType::Pwsh),
            _ => Err(ShellError::UnsupportedShellType(value.to_string())),
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Bash => write!(f, "bash"),
            ShellType::Pwsh => write!(f, "powershell"),
        }
    }
}

impl ShellType {
    fn command_name(&self) -> &str {
        match self {
            ShellType::Zsh => "zsh",
            ShellType::Bash => "bash",
            ShellType::Pwsh => "pwsh",
        }
    }

    fn history_file_path(&self) -> PathBuf {
        match self {
            ShellType::Zsh | ShellType::Bash => {
                let home_dir = home_dir().unwrap();
                let history_file_name = match self {
                    ShellType::Zsh => ".zsh_history",
                    ShellType::Bash => ".bash_history",
                    _ => unreachable!(),
                };
                home_dir.join(history_file_name)
            }
            ShellType::Pwsh => {
                let history_path = Command::new("pwsh")
                    .arg("-Command")
                    .arg("(Get-PSReadlineOption).HistorySavePath")
                    .output()
                    // TODO: Handle error.
                    .unwrap();
                let history_path = String::from_utf8_lossy(&history_path.stdout);
                let history_path = history_path.trim();
                PathBuf::from(history_path)
            }
        }
    }
}

#[derive(Debug)]
pub struct Shell;

impl Shell {
    pub fn extract_env_info() -> Result<SystemInfo, ShellError> {
        let os = sysinfo::System::name().ok_or(ShellError::FailedToExtractSystemInfo("Failed to get system name".to_string()))?;
        let os_version =
            sysinfo::System::os_version().ok_or(ShellError::FailedToExtractSystemInfo("Failed to extract OS version".to_string()))?;
        let arch =
            sysinfo::System::cpu_arch().ok_or(ShellError::FailedToExtractSystemInfo("Failed to get CPU architecture".to_string()))?;
        let shell_type = Self::current_shell_type()?;
        Ok(SystemInfo {
            shell: shell_type.to_string(),
            os,
            os_version,
            arch,
        })
    }

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
            ShellType::Pwsh => {
                let history_path = shell_type.history_file_path();
                let mut child = Command::new("pwsh")
                    .arg("-Command")
                    .arg(format!(
                        "Add-Content -Path \"{}\" -Value \"{}\"",
                        history_path.to_str().unwrap(),
                        command
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
            ShellType::Zsh | ShellType::Bash | ShellType::Pwsh => {
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
        // TODO: Support PowerShell 5.1 and below. This assumes PowerShell Core.
        if std::env::var("PSModulePath").is_ok() {
            return Ok(ShellType::Pwsh);
        }

        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Ok(ShellType::Zsh);
            } else if shell.contains("bash") {
                return Ok(ShellType::Bash);
            }
        }
        Err(ShellError::FailedToExtractSystemInfo("Failed to get shell type".to_string()))
    }
}
