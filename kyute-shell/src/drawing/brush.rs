//! Brushes.
use crate::drawing::target::RenderTarget;
use crate::drawing::{mk_color_f, mk_matrix_3x2, Color, Transform};
use std::ptr;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use wio::com::ComPtr;

pub trait Brush {
    fn as_raw_brush(&self) -> *mut ID2D1Brush;
}

#[derive(Clone, Debug)]
pub struct SolidColorBrush(pub(crate) ComPtr<ID2D1SolidColorBrush>);

impl SolidColorBrush {
    pub fn new(target: &RenderTarget, color: Color) -> SolidColorBrush {
        unsafe {
            let mut brush = ptr::null_mut();
            let brush_props = D2D1_BRUSH_PROPERTIES {
                opacity: 1.0, // FIXME what's the difference with color.a?
                transform: mk_matrix_3x2(&Transform::identity()),
            };
            let hr =
                target
                    .target
                    .CreateSolidColorBrush(&mk_color_f(color), &brush_props, &mut brush);
            assert!(SUCCEEDED(hr));
            SolidColorBrush(ComPtr::from_raw(brush))
        }
    }
}

impl Brush for SolidColorBrush {
    fn as_raw_brush(&self) -> *mut ID2D1Brush {
        self.0.as_raw().cast()
    }
}

#[derive(Clone, Debug)]
pub struct LinearGradientBrush(pub(crate) ComPtr<ID2D1LinearGradientBrush>);

impl LinearGradientBrush {}

impl Brush for LinearGradientBrush {
    fn as_raw_brush(&self) -> *mut ID2D1Brush {
        self.0.as_raw().cast()
    }
}

#[derive(Clone, Debug)]
pub struct RadialGradientBrush(pub(crate) ComPtr<ID2D1RadialGradientBrush>);

impl RadialGradientBrush {}

impl Brush for RadialGradientBrush {
    fn as_raw_brush(&self) -> *mut ID2D1Brush {
        self.0.as_raw().cast()
    }
}

pub trait IntoBrush {
    type Brush: Brush;
    fn into_brush(self, target: &RenderTarget) -> Self::Brush;
}

impl IntoBrush for Color {
    type Brush = SolidColorBrush;

    fn into_brush(self, target: &RenderTarget) -> Self::Brush {
        SolidColorBrush::new(target, self)
    }
}

impl<T: Brush> IntoBrush for T {
    type Brush = T;
    fn into_brush(self, _target: &RenderTarget) -> T {
        self
    }
}