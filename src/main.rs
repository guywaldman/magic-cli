mod cli;
mod engine;
mod ollama;

use cli::Cli;

fn main() {
    let cli = Cli::new();
    cli.run(std::env::args()).unwrap();
}
