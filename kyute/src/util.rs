use miniqt_sys::util::{Deletable, Upcast};
use std::ptr::NonNull;

mod cbox;
mod ptr;

pub use cbox::CBox;
pub use ptr::Ptr;

/// Wrapper around a raw pointer that *may* own the object it points to.
///
/// Typically used to track ownership of Qt widgets as they are parented to other widgets.
pub(crate) struct MaybeOwned<T: Deletable + ?Sized> {
    /// Raw pointer to the object.
    ptr: NonNull<T>,
    /// Whether we own the object and should run its destructor when we drop.
    owned: bool,
}

impl<T: Deletable + ?Sized> MaybeOwned<T> {
    #[inline]
    pub fn owned(ptr: NonNull<T>) -> MaybeOwned<T> {
        MaybeOwned { ptr, owned: true }
    }

    #[inline]
    pub fn unowned(ptr: NonNull<T>) -> MaybeOwned<T> {
        MaybeOwned { ptr, owned: false }
    }

    #[inline]
    pub fn is_owned(&self) -> bool {
        return self.owned;
    }

    /// Transfers ownership of the pointer to the caller. The caller becomes responsible for
    /// deleting the object.
    pub fn disown<U>(&mut self) -> Option<NonNull<U>>
    where
        U: Deletable,
        T: Upcast<U>,
    {
        if !self.owned {
            return None;
        }
        self.owned = false;
        unsafe {
            Some(NonNull::new_unchecked(Upcast::<U>::upcast(
                self.ptr.as_ptr(),
            )))
        }
    }

    #[inline]
    pub(crate) fn as_ptr<U>(&self) -> NonNull<U>
    where
        T: Upcast<U>,
    {
        unsafe { NonNull::new_unchecked(Upcast::<U>::upcast(self.ptr.as_ptr())) }
    }

    /*pub(crate) fn upcast<U: Deletable>(&self) -> *mut U
    where
    {
        Inherits::<U>::upcast(self.as_raw())
    }*/
}

impl<T: Deletable + ?Sized> Drop for MaybeOwned<T> {
    fn drop(&mut self) {
        if self.owned {
            // TODO safety?
            unsafe { Deletable::delete(self.ptr.as_ptr()) }
        }
    }
}
