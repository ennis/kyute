//#![allow(clippy::all)]
#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    missing_copy_implementations,
    non_upper_case_globals
)]

use std::mem::MaybeUninit;
use std::os::raw::c_char;

#[macro_use]
pub mod util;
mod ffi;
pub use ffi::*;

// drop impls
impl_drop!(QString, QString_destructor);
impl_drop!(QVariant, QVariant_destructor);

// deletable impls
impl_deletable!(QPushButton; QPushButton_delete);
impl_deletable!(QWidget; QWidget_delete);
impl_deletable!(QObject; QObject_delete);
impl_deletable!(QLineEdit; QLineEdit_delete);
impl_deletable!(QComboBox; QComboBox_delete);
impl_deletable!(QLabel; QLabel_delete);

// inheritance relationships
impl_multiple_inheritance!(QWidget: QObject;      UPCAST QWidget_upcast_QObject;      DOWNCAST QObject_downcast_QWidget);
impl_multiple_inheritance!(QWidget: QPaintDevice; UPCAST QWidget_upcast_QPaintDevice; DOWNCAST QPaintDevice_downcast_QWidget);

impl_inheritance!(QCoreApplication: QObject);
impl_inheritance!(QGuiApplication: QObject);
impl_inheritance!(QGuiApplication: QCoreApplication);
impl_inheritance!(QApplication: QObject);
impl_inheritance!(QApplication: QCoreApplication);
impl_inheritance!(QApplication: QGuiApplication);

impl_inheritance!(QAbstractButton: QWidget);
impl_inheritance!(QAbstractButton: QObject);

impl_inheritance!(QPushButton: QAbstractButton);
impl_inheritance!(QPushButton: QWidget);
impl_inheritance!(QPushButton: QObject);

impl_inheritance!(QCheckBox: QAbstractButton);
impl_inheritance!(QCheckBox: QWidget);
impl_inheritance!(QCheckBox: QObject);

impl_inheritance!(QLineEdit: QWidget);
impl_inheritance!(QLineEdit: QObject);

impl_inheritance!(QLayout: QObject);
impl_inheritance!(QBoxLayout: QLayout);
impl_inheritance!(QVBoxLayout: QBoxLayout);
impl_inheritance!(QHBoxLayout: QBoxLayout);
impl_inheritance!(QVBoxLayout: QLayout);
impl_inheritance!(QHBoxLayout: QLayout);
impl_inheritance!(QFormLayout: QLayout);

impl_inheritance!(QComboBox: QWidget);
impl_inheritance!(QComboBox: QObject);

impl_inheritance!(QLabel: QWidget);

impl_inheritance!(QLinearGradient: QGradient);

impl Clone for QColor {
    fn clone(&self) -> Self {
        // easier than trying to make bindgen derive copy+clone for *just one* class
        unsafe { std::mem::transmute_copy(self) }
    }
}

impl Copy for QColor {}

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

impl<S: AsRef<str>> From<S> for QString {
    fn from(s: S) -> Self {
        let s = s.as_ref();
        unsafe {
            let mut out = MaybeUninit::<QString>::uninit();
            QString_constructor(out.as_mut_ptr());
            QString_fromUtf8(
                s.as_ptr() as *const c_char,
                s.len() as i32,
                out.as_mut_ptr(),
            );
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
