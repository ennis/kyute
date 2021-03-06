use crate::error::Error;
use thiserror::Error;

/// Windows backend error type.
#[derive(Error, Debug)]
pub enum PlatformError {
    /// HRESULT error type during execution of a command.
    #[error("OS error")]
    WindowsApiError(#[from] windows::core::Error),
    /// Winit-issued error
    #[error("winit error")]
    Winit(#[from] winit::error::OsError),
}

impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Self {
        Error::Platform(PlatformError::WindowsApiError(err))
    }
}
