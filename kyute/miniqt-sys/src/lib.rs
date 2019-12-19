//#![allow(clippy::all)]
#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    missing_copy_implementations,
    non_upper_case_globals
)]

use std::os::raw::c_char;
use std::mem::MaybeUninit;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// unpin impls

//impl !Unpin for QString {}
//impl !Unpin for QObject {}
//impl !Unpin for QWidget {}

// drop impls

impl Clone for QColor {
    fn clone(&self) -> Self {
        // easier than trying to make bindgen derive copy+clone for *just one* class
        unsafe {
            std::mem::transmute_copy(self)
        }
    }
}

impl Copy for QColor {}

macro_rules! impl_drop {
    ($t:ty, $f:ident) => {
        impl Drop for $t {
            fn drop(&mut self) {
                unsafe {
                    $f(self)
                }
            }
        }
    };
}

impl_drop!(QString, QString_destructor);
impl_drop!(QVariant, QVariant_destructor);

impl ToString for QString {
    fn to_string(&self) -> String {
        unsafe {
            let utf16 = QString_utf16(self);
            let len = QString_size(self);
                String::from_utf16(std::slice::from_raw_parts(utf16, len as usize))
                    .expect("text was not valid utf-16")
        }
    }
}

impl From<&str> for QString {
    fn from(s: &str) -> Self {
        unsafe {
            let mut out = MaybeUninit::<QString>::uninit();
            QString_constructor(out.as_mut_ptr());
            QString_fromUtf8(s.as_ptr() as *const c_char, s.len() as i32, out.as_mut_ptr());
            out.assume_init()
        }
    }
}

impl From<u64> for QVariant {
    fn from(v: u64) -> QVariant {
        unsafe {
            let mut out = MaybeUninit::<QVariant>::uninit();
            QVariant_constructor_quint64(out.as_mut_ptr(), v);
            out.assume_init()
        }
    }
}

impl QRectF {
    pub fn from_xywh(x: f64, y: f64, w: f64, h: f64) -> QRectF {
        let mut r = MaybeUninit::<QRectF>::uninit();
        unsafe {
            QRectF_constructor(r.as_mut_ptr(), x, y, w, h);
            r.assume_init()
        }
    }
}

