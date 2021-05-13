use std::{error, fmt};

/// Errors emitted.
pub enum Error {
    /// HRESULT error type during execution of a command.
    WindowsApiError(windows::Error),
    /// Winit-issued error
    Winit(winit::error::OsError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::WindowsApiError(err) => fmt::Display::fmt(&err, f),
            Error::Winit(os) => fmt::Display::fmt(&os, f),
        }
    }
}

impl error::Error for Error {}

impl From<windows::Error> for Error {
    fn from(err: windows::Error) -> Self {
        Error::WindowsApiError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
