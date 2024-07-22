use std::error::Error;

use async_trait::async_trait;

use super::config::MagicCliConfig;

#[derive(Debug, Clone)]
pub struct MagicCliRunOptions {
    pub config: MagicCliConfig,
}

#[async_trait]
pub trait MagicCliSubcommand {
    async fn run(&self, options: MagicCliRunOptions) -> Result<(), Box<dyn Error>>;
}
