mod cli;
mod core;
mod db;
mod ollama;

use cli::MagicCli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = MagicCli;
    let args: Vec<String> = std::env::args().collect();
    cli.run(&args)?;
    Ok(())
}
