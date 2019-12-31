use crate::util::Ptr;
use miniqt_sys::util::{Deletable, Upcast};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Wrapper around a raw pointer that owns the object it points to. The equivalent of `Box` for
/// `Deletable` types.
///
/// The destructor of the object is run as a part of the `Drop` implementation for this type.
pub struct CBox<T: Deletable + ?Sized>(NonNull<T>);

impl<T: Deletable + ?Sized> CBox<T> {
    /// Wraps the raw pointer.
    pub unsafe fn new(ptr: *mut T) -> CBox<T> {
        CBox(NonNull::new(ptr).expect("null pointer"))
    }

    #[inline]
    pub const unsafe fn from_non_null(ptr: NonNull<T>) -> CBox<T> {
        CBox(ptr)
    }

    #[inline]
    pub const unsafe fn from_ptr(ptr: Ptr<T>) -> CBox<T> {
        CBox::from_non_null(ptr.as_non_null())
    }

    #[inline]
    pub const fn as_ptr(&self) -> Ptr<T> {
        Ptr::from_non_null(self.0)
    }

    #[inline]
    pub const fn as_raw_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub fn into_raw(self) -> NonNull<T> {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }

    #[inline]
    pub fn upcast<U: Deletable + ?Sized>(self) -> CBox<U>
    where
        T: Upcast<U>,
    {
        unsafe { CBox::from_non_null(self.as_ptr().upcast().as_non_null()) }
    }
}

impl<T: Deletable + ?Sized> Deref for CBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*(self.0.as_ptr()) }
    }
}

impl<T: Deletable + ?Sized> DerefMut for CBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl<T: Deletable + ?Sized> Drop for CBox<T> {
    fn drop(&mut self) {
        unsafe { <T as Deletable>::delete(self.0.as_ptr()) }
    }
}
