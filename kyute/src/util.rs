//! Utilities for managing opaque FFI types.
//!
//! This module contains utility traits to manage opaque FFI types: this includes traits for
//! deallocation, running destructors associated to a type, and, for C++ classes,
//! upcasting or downcasting pointers to base or derived class types.
//!
//! Objects of opaque FFI types are allocated externally (via, e.g. `operator new` in C++).
//! They are accessed in rust through raw `*mut T` pointers, without any lifetime checks or RAII.
//! Most of the functions in this module are thus unsafe.
//!
//! This module is used internally for managing the Qt types exposed by `miniqt_sys`.
use miniqt_sys::*;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;

/// Trait that specifies the destructor and deallocation function for an opaque FFI type.
pub trait Deletable {
    /// Runs the destructor of the object and releases the memory allocated for the object.
    ///
    /// Usually this just calls a wrapper for the C++ `operator delete`.
    unsafe fn delete(obj: *mut Self);
}

macro_rules! impl_deletable {
    ($t:ty; $f:path) => {
        impl crate::util::Deletable for $t {
            unsafe fn delete(obj: *mut Self) {
                $f(obj as *mut _)
            }
        }
    };
}

/// Indicates that a C++ class type inherits from another C++ class type `U`.
pub unsafe trait Inherits<U: ?Sized> {
    /// Casts this pointer to a pointer of the base class type.
    fn upcast(ptr: *mut Self) -> *mut U;

    /// Casts this pointer to a pointer of the base class type.
    fn upcast_const(ptr: *const Self) -> *const U {
        Self::upcast(ptr as *mut Self) as *const U
    }

    /// Downcasts a pointer of the base class type to a pointer of this (derived) type.
    ///
    /// UB if the provided pointer does not point to an instance of the derived type.
    unsafe fn downcast_unchecked(ptr: *mut U) -> *mut Self;
}

/// The `Inherits` relation is reflexive.
unsafe impl<T: ?Sized> Inherits<T> for T {
    #[inline]
    fn upcast(ptr: *mut T) -> *mut T {
        ptr
    }

    #[inline]
    unsafe fn downcast_unchecked(ptr: *mut T) -> *mut T {
        ptr
    }
}

/// Wrapper around a raw pointer that *may* own the object it points to.
///
/// Typically used to track ownership of Qt widgets as they are parented to other widgets.
pub(crate) struct MaybeOwned<T: Deletable + ?Sized> {
    /// Raw pointer to the object.
    ptr: *mut T,
    /// Whether we own the object and should run its destructor when we drop.
    owned: bool,
}

impl<T: Deletable + ?Sized> MaybeOwned<T> {
    #[inline]
    pub fn owned(ptr: *mut T) -> MaybeOwned<T> {
        MaybeOwned { ptr, owned: true }
    }

    #[inline]
    pub fn unowned(ptr: *mut T) -> MaybeOwned<T> {
        MaybeOwned {
            ptr,
            owned: false
        }
    }

    #[inline]
    pub fn is_owned(&self) -> bool {
        return self.owned
    }

    /// Transfers ownership of the pointer to the caller. The caller becomes responsible for
    /// deleting the object.
    pub fn disown<U>(&mut self) -> *mut U
        where
            U: Deletable,
            T: Inherits<U>,
    {
        self.owned = false;
        Inherits::<U>::upcast(self.ptr)
    }

    #[inline]
    pub(crate) fn as_raw<U>(&self) -> *mut U
        where
            T: Inherits<U>,
    {
        Inherits::<U>::upcast(self.ptr)
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
            // TODO safety
            unsafe { Deletable::delete(self.ptr) }
        }
    }
}

/// Wrapper around a raw pointer that owns the object it points to. The equivalent of `Box` for
/// `Deletable` types.
///
/// The destructor of the object is run as a part of the `Drop` implementation for this type.
pub struct CBox<T: Deletable + ?Sized>(*mut T);

impl<T: Deletable + ?Sized> CBox<T> {
    /// Wraps the raw
    pub unsafe fn new(ptr: *mut T) -> CBox<T> {
        CBox(ptr)
    }

    /*pub fn as_raw(&self) -> *const T {
        self.0
    }

    pub fn as_mut_raw(&self) -> *mut T {
        self.0
    }*/

    #[inline]
    pub fn into_raw(self) -> *mut T {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }

    /*pub fn upcast<U: Deletable>(self) -> CBox<U>
    where
        T: Inherits<U>,
    {
        let ptr = Inherits::upcast(self.as_mut_ptr());
        mem::forget(self);
        CBox(ptr)
    }

    pub unsafe fn cast<U: Deletable>(self) -> CBox<U> {
        let new = CBox::new(self.as_mut_ptr() as *mut U);
        mem::forget(self);
        new
    }*/
}

impl<T: Deletable + ?Sized> Deref for CBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*(self.0 as *const T) }
    }
}

impl<T: Deletable + ?Sized> DerefMut for CBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<T: Deletable + ?Sized> Drop for CBox<T> {
    fn drop(&mut self) {
        unsafe { <T as Deletable>::delete(self.0) }
    }
}

/// Allows converting a `*mut Derived` into a `*mut Base` when `Derived: Inherits<Base>`.
/// Use with caution, make sure that the corresponding C++ types do not use multiple inheritance.
macro_rules! impl_inherits {
    ($derived:ty: $base:ty) => {
        unsafe impl $crate::util::Inherits<$base> for $derived {
            #[inline]
            fn upcast(ptr: *mut Self) -> *mut $base {
                // this assumes no multiple inheritance shenanigans on the c++ side
                // TODO call a c++ stub to do a proper static_cast
                ptr as *mut $base
            }

            #[inline]
            unsafe fn downcast_unchecked(ptr: *mut $base) -> *mut Self {
                ptr as *mut Self
            }
        }
    };
}

macro_rules! impl_inherits_multi {
    ($derived:ty: $base:ty; UPCAST $upcast_fn:path; DOWNCAST $downcast_fn:path) => {
        unsafe impl $crate::util::Inherits<$base> for $derived {
            #[inline]
            fn upcast(ptr: *mut Self) -> *mut $base {
                unsafe {
                    $upcast_fn(ptr)
                }
            }

            #[inline]
            unsafe fn downcast_unchecked(ptr: *mut $base) -> *mut Self {
                $downcast_fn(ptr)
            }
        }
    };
}

// Inherits and deletable impls
// Q: Should this be in the -sys crate?

impl_inherits_multi!(QWidget: QObject;      UPCAST QWidget_upcast_QObject;      DOWNCAST QObject_downcast_QWidget);
impl_inherits_multi!(QWidget: QPaintDevice; UPCAST QWidget_upcast_QPaintDevice; DOWNCAST QPaintDevice_downcast_QWidget);
impl_deletable!(QWidget; QWidget_delete);
impl_deletable!(QObject; QObject_delete);
