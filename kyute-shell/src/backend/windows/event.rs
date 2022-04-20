//! Wrapper around `CreateEvent`
use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// A Win32 event object.
pub(crate) struct Win32Event {
    handle: HANDLE,
}

impl Drop for Win32Event {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Win32Event {
    pub(crate) unsafe fn from_raw(handle: HANDLE) -> Win32Event {
        Win32Event { handle }
    }

    pub(crate) fn handle(&self) -> HANDLE {
        self.handle
    }
}
