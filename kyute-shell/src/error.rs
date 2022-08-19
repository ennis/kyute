use crate::backend;
use thiserror::Error;

/// Errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("backend error")]
    Platform(#[from] backend::PlatformError),
    #[error("window was closed")]
    WindowClosed,
}

pub type Result<T> = std::result::Result<T, Error>;
