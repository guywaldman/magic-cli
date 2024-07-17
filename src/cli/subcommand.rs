use std::error::Error;

pub trait MagicCliSubcommand {
    fn run(&self) -> Result<(), Box<dyn Error>>;
}
