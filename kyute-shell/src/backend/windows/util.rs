use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

// --- this section taken from https://github.com/linebender/druid/blob/f588fa76bc88215ce6b2b500d0eba26149ca8368/druid-shell/src/backend/windows/util.rs#L63
// see licenses

pub trait ToWide {
    fn to_wide_sized(&self) -> Vec<u16>;
    fn to_wide(&self) -> Vec<u16>;
}

impl<T> ToWide for T
where
    T: AsRef<OsStr>,
{
    fn to_wide_sized(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}
