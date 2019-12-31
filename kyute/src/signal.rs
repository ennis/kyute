//! Signal/slot utilities.
//!
//! We can't directly connect a rust function to a Qt signal. Instead, we connect the signal to
//! a proxy QObject with a slot that calls our callback.

use crate::util::{CBox, Ptr};
use miniqt_sys::util::Upcast;
use miniqt_sys::*;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_int;

/// Equivalent to the SIGNAL macro
macro_rules! qt_signal {
    ($name:tt) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!("2", $name, "\0").as_bytes())
        }
    };
}

/// Equivalent to the SLOT macro
macro_rules! qt_slot {
    ($name:tt) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!("1", $name, "\0").as_bytes())
        }
    };
}

pub unsafe trait SignalArgument {
    const SLOT_NAME: &'static CStr;
    const CALLBACK_CREATE_FN: unsafe extern "C" fn(
        usize,
        usize,
        Option<unsafe extern "C" fn(usize, usize, Self)>,
    ) -> *mut QObject;
}

macro_rules! impl_signal_argument {
    ($argty:ty; $cb_create_fn:path; $slot:tt) => {
        unsafe impl SignalArgument for $argty {
            const SLOT_NAME: &'static CStr = qt_slot!($slot);
            const CALLBACK_CREATE_FN: unsafe extern "C" fn(
                usize,
                usize,
                Option<unsafe extern "C" fn(usize, usize, Self)>,
            ) -> *mut QObject = $cb_create_fn;
        }
    };
}

impl_signal_argument!(c_int; MQCallback_int_new; "trigger(int)");
impl_signal_argument!(*const QString; MQCallback_QString_new; "trigger(const QString&)");

//--------------------------------------------------------------------------------------------------

/// A callback bound to a Qt signal.
pub struct Slot<'a> {
    cb: Box<dyn FnMut() + 'a>,
    receiver: CBox<QObject>,
}

/// A one-parameter callback bound to a Qt signal.
pub struct Slot1<'a, T: SignalArgument> {
    cb: Box<dyn FnMut(T) + 'a>,
    receiver: CBox<QObject>,
}

/// "Landing pad" for FFI callbacks.
///
/// All parameter-less callbacks from the C++ side call this function. It reconstructs the original
/// closure object from the two data values passed during registration (returned by
/// [Callback::data]) and invokes the closure.
unsafe extern "C" fn landing_pad_void(data0: usize, data1: usize) {
    let ptr: *mut dyn FnMut() = mem::transmute(std::raw::TraitObject {
        data: data0 as *mut (),
        vtable: data1 as *mut (),
    });

    (&mut *ptr)();
}

/// "Landing pad" for FFI callbacks with one parameter.
///
/// See [landing_pad_void].
unsafe extern "C" fn landing_pad_1<T>(data0: usize, data1: usize, value: T) {
    let ptr: *mut dyn FnMut(T) = mem::transmute(std::raw::TraitObject {
        data: data0 as *mut (),
        vtable: data1 as *mut (),
    });

    (&mut *ptr)(value);
}

//--------------------------------------------------------------------------------------------------

impl<'a> Slot<'a> {
    /// TODO is it safe?
    pub fn new<F>(cb: F) -> Slot<'a>
    where
        F: FnMut() + 'a,
    {
        unsafe {
            let cb: Box<dyn FnMut()> = Box::new(cb);
            let std::raw::TraitObject { data, vtable } = mem::transmute(cb.as_ref());
            let receiver = CBox::new(MQCallback_new(
                data as usize,
                vtable as usize,
                Some(landing_pad_void),
            ));
            Slot { receiver, cb }
        }
    }

    /// TODO could it be actually safe? (does it disconnects on drop?)
    pub unsafe fn connect<S>(&mut self, sender: Ptr<S>, signal: &CStr)
    where
        S: Upcast<QObject>,
    {
        QObject_connect_abi(
            sender.upcast().as_ptr(),
            signal.as_ptr(),
            &*self.receiver,
            qt_slot!("trigger()").as_ptr(),
            Qt_ConnectionType_AutoConnection,
        );
    }
}

impl<'a, T: SignalArgument> Slot1<'a, T> {
    pub fn new<F>(cb: F) -> Slot1<'a, T>
    where
        F: FnMut(T) + 'a,
    {
        unsafe {
            let cb: Box<dyn FnMut(T)> = Box::new(cb);
            let std::raw::TraitObject { data, vtable } = mem::transmute(cb.as_ref());
            let receiver = CBox::new(T::CALLBACK_CREATE_FN(
                data as usize,
                vtable as usize,
                Some(landing_pad_1::<T>),
            ));
            Slot1 { receiver, cb }
        }
    }

    pub unsafe fn connect<S>(&mut self, sender: Ptr<S>, signal: &CStr)
    where
        S: Upcast<QObject>,
    {
        QObject_connect_abi(
            sender.upcast().as_ptr(),
            signal.as_ptr(),
            &*self.receiver,
            T::SLOT_NAME.as_ptr(),
            Qt_ConnectionType_AutoConnection,
        );
    }
}

/*
macro_rules! __define_callback_1 {
    ($name:ident, $arg_ty:ty, $callback_create_fn:path [ $slot:tt ]) => {
        pub unsafe fn $name<'a, S, CB>(
            sender: *const S,
            signal: &CStr,
            cb: CB,
        ) -> Callback1<'a, $arg_ty>
        where
            S: Inherits<QObject>,
            CB: FnMut($arg_ty) + 'a,
        {
            let cb: Box<dyn FnMut($arg_ty)> = Box::new(cb);
            let std::raw::TraitObject { data, vtable } = mem::transmute(cb.as_ref());
            let receiver = CBox::new($callback_create_fn(
                data as usize,
                vtable as usize,
                Some(landing_pad_1::<$arg_ty>),
            ));
            QObject_connect_abi(
                Inherits::upcast(sender as *mut S) as *const QObject,
                signal.as_ptr(),
                &*receiver,
                qt_slot!($slot).as_ptr(),
                Qt_ConnectionType_AutoConnection,
            );
            Callback1 { receiver, cb }
        }
    };
}

__define_connect_1_fn!(
    qt_connect_callback_int,
    std::os::raw::c_int,
    MQCallback_int_new["trigger(int)"]
);
__define_connect_1_fn!(
    qt_connect_callback_qstring,
    *const QString,
    MQCallback_QString_new["trigger(const QString&)"]
);
*/
