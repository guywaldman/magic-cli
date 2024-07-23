use std::error::Error;

use async_trait::async_trait;

use super::config::MagicCliConfigManager;

#[derive(Debug, Clone)]
pub struct MagicCliRunOptions {
    pub config: MagicCliConfigManager,
}

#[async_trait]
pub trait MagicCliSubcommand {
    async fn run(&self, options: MagicCliRunOptions) -> Result<(), Box<dyn Error>>;
}
