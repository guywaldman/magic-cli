mod cli;
mod engine;
mod ollama;

use cli::{Cli, CliConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CliConfig::load_config()?;
    let cli = Cli::new(config);
    cli.run(std::env::args()).unwrap();
    Ok(())
}
