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
pub unsafe trait Upcast<U: ?Sized> {
    /// Casts this pointer to a pointer of the base class type.
    fn upcast(ptr: *mut Self) -> *mut U;

    /// Casts this pointer to a pointer of the base class type.
    fn upcast_const(ptr: *const Self) -> *const U {
        Self::upcast(ptr as *mut Self) as *const U
    }

    /*/// Downcasts a pointer of the base class type to a pointer of this (derived) type.
    ///
    /// UB if the provided pointer does not point to an instance of the derived type.
    unsafe fn downcast_unchecked(ptr: *mut U) -> *mut Self;*/
}

/// The `Inherits` relation is reflexive.
unsafe impl<T: ?Sized> Upcast<T> for T {
    #[inline]
    fn upcast(ptr: *mut T) -> *mut T {
        ptr
    }

    /*#[inline]
    unsafe fn downcast_unchecked(ptr: *mut T) -> *mut T {
        ptr
    }*/
}

/// Allows converting a `*mut Derived` into a `*mut Base` when `Derived: Inherits<Base>`.
/// Use with caution, make sure that the corresponding C++ types do not use multiple inheritance.
macro_rules! impl_inheritance {
    ($derived:ty: $base:ty) => {
        unsafe impl $crate::util::Upcast<$base> for $derived {
            #[inline]
            fn upcast(ptr: *mut Self) -> *mut $base {
                // this assumes no multiple inheritance shenanigans on the c++ side
                // TODO call a c++ stub to do a proper static_cast
                ptr as *mut $base
            }

            /* #[inline]
            unsafe fn downcast_unchecked(ptr: *mut $base) -> *mut Self {
                ptr as *mut Self
            }*/
        }
    };
}

macro_rules! impl_multiple_inheritance {
    ($derived:ty: $base:ty; UPCAST $upcast_fn:path; DOWNCAST $downcast_fn:path) => {
        unsafe impl $crate::util::Upcast<$base> for $derived {
            #[inline]
            fn upcast(ptr: *mut Self) -> *mut $base {
                unsafe { $upcast_fn(ptr) }
            }

            /*#[inline]
            unsafe fn downcast_unchecked(ptr: *mut $base) -> *mut Self {
                $downcast_fn(ptr)
            }*/
        }
    };
}

macro_rules! impl_drop {
    ($t:ty, $f:ident) => {
        impl Drop for $t {
            fn drop(&mut self) {
                unsafe { $f(self) }
            }
        }
    };
}
