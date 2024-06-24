use colored::Colorize;
use std::process::Command;

#[derive(Debug)]
pub struct Shell;

impl Shell {
    pub fn add_command_to_history(command: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if current shell is bash
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        if shell.contains("zsh") {
            // Append to ~/.zsh_history
            let mut child = Command::new("zsh")
                .arg("-c")
                .arg(format!("echo \"{}\" >> ~/.zsh_history", command))
                .spawn()
                .expect("Failed to execute command");
            let resp = child.wait()?;

            if !resp.success() {
                println!(
                    "{}",
                    format!("Failed to add command to shell history. Error: {}", resp.code().unwrap())
                        .red()
                        .bold()
                );
            } else {
                println!("{}", "Command added to shell history".green().bold());
            }
        } else {
            eprintln!("{}", "Unsupported shell, only zsh is currently supported".red().bold());
        }

        Ok(())
    }
}
