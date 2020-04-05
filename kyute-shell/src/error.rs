use std::{error, fmt};
use winapi::shared::winerror::{HRESULT, SUCCEEDED};

/// Errors emitted.
pub enum Error {
    /// HRESULT error type during execution of a command.
    HResultError(HRESULT),
    /// DirectX-OpenGL interop error
    OpenGlInteropError,
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
            Error::HResultError(hr) => write!(f, "[HRESULT {:08X}]", hr),
            Error::OpenGlInteropError => write!(
                f,
                "Unspecified OpenGL/DirectX interop error (WGL_NV_DX_interop2)"
            ),
            Error::Winit(os) => fmt::Display::fmt(&os, f),
        }
    }
}

impl error::Error for Error {}

impl From<HRESULT> for Error {
    fn from(hr: HRESULT) -> Self {
        Error::HResultError(hr)
    }
}

/*/// Checks that `SUCCEEDED(hr)` is true, otherwise returns an `Err(HResultError(hr))`;
pub(crate) fn check_hr(hr: HRESULT) -> Result<HRESULT> {
    if !SUCCEEDED(hr) {
        Err(Error::HResultError(hr))
    } else {
        Ok(hr)
    }
}*/

pub(crate) fn wrap_hr<R, F: FnOnce() -> R>(hr: HRESULT, f: F) -> Result<R> {
    if !SUCCEEDED(hr) {
        Err(Error::HResultError(hr))
    } else {
        Ok(f())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
