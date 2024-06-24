mod cli;
mod engine;
mod ollama;

use cli::MagicCli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = MagicCli;
    cli.run(std::env::args())?;
    Ok(())
}
