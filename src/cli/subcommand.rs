use std::error::Error;

use async_trait::async_trait;

#[async_trait]
pub trait MagicCliSubcommand {
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}
