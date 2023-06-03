//! Platform-specific implementations of certain types and functions.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;
