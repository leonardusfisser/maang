use thiserror::Error;
use std::error::Error as StdError;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("failed to load env from {path}: {source}")]
    Load {
        path: String,
        #[source] source: Box<dyn StdError + Send + Sync>,
    },
    #[error("env var {0} missing")]
    MissingVar(String),
}
