mod cli;
mod core;
mod lm;

use cli::MagicCli;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = MagicCli;
    let args: Vec<String> = std::env::args().collect();
    match cli.run(&args).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", format!("ERROR: {}", err).red().bold());
            std::process::exit(1);
        }
    }
    Ok(())
}
