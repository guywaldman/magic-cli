use std::error::Error;

use colored::Colorize;

use crate::core::Shell;

use super::subcommand::MagicCliSubcommand;

pub struct SysInfoSubcommand;

impl SysInfoSubcommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl MagicCliSubcommand for SysInfoSubcommand {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let sysinfo = Shell::extract_env_info()?;
        println!("System information as detected by the CLI:");
        println!();
        println!("{}: {}", "OS".dimmed(), sysinfo.os.to_string().blue().bold());
        println!("{}: {}", "OS version".dimmed(), sysinfo.os_version.to_string().blue().bold());
        println!("{}: {}", "CPU architecture".dimmed(), sysinfo.arch.to_string().blue().bold());
        println!("{}: {}", "Shell".dimmed(), sysinfo.shell.to_string().blue().bold());
        Ok(())
    }
}
