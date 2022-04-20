use crate::backend;
use thiserror::Error;

/// Errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("backend error")]
    Platform(#[from] backend::PlatformError),
}

pub type Result<T> = std::result::Result<T, Error>;
