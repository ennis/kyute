use miniqt_sys::util::Upcast;
use std::cmp::Ordering;
use std::fmt;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Ptr<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> Ptr<T> {
    /// Panics if pointer is null.
    pub fn new(raw: *mut T) -> Ptr<T> {
        Ptr(NonNull::new(raw).expect("null pointer"))
    }

    #[inline]
    pub const fn from_non_null(ptr: NonNull<T>) -> Ptr<T> {
        Ptr(ptr)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub const fn as_non_null(self) -> NonNull<T> {
        self.0
    }

    #[inline]
    pub fn upcast<U: ?Sized>(self) -> Ptr<U>
    where
        T: Upcast<U>,
    {
        Ptr(unsafe { NonNull::new_unchecked(Upcast::upcast(self.as_ptr())) })
    }
}

/*impl<U: ?Sized, T: ?Sized + Upcast<U>> From<Ptr<T>> for Ptr<U> {
    #[inline]
    fn from(derived: Ptr<T>) -> Self {
        derived.upcast()
    }
}*/

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Ptr(self.0.clone())
    }
}

impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> fmt::Debug for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: ?Sized> fmt::Pointer for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: ?Sized> Eq for Ptr<T> {}

impl<T: ?Sized> PartialEq for Ptr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<T: ?Sized> Ord for Ptr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl<T: ?Sized> PartialOrd for Ptr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}
