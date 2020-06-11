//! Brushes.
use crate::drawing::context::DrawContext;
use crate::drawing::{
    mk_color_f, mk_matrix_3x2, mk_point_f, Color, GradientStopCollection, Point, Transform,
};
use palette::{Alpha, LinSrgb, LinSrgba, Srgb};
use std::ptr;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use wio::com::ComPtr;

pub(crate) enum BrushImpl {
    SolidColor(ComPtr<ID2D1SolidColorBrush>),
    RadialGradient(ComPtr<ID2D1RadialGradientBrush>),
    LinearGradient(ComPtr<ID2D1LinearGradientBrush>),
}

/// Brushes to fill or stroke geometry.
pub struct Brush(pub(crate) BrushImpl);

impl Brush {
    pub fn new_solid_color(ctx: &DrawContext, color: Color) -> Brush {
        unsafe {
            let mut brush = ptr::null_mut();
            let brush_props = D2D1_BRUSH_PROPERTIES {
                opacity: 1.0, // FIXME what's the difference with color.a?
                transform: mk_matrix_3x2(&Transform::identity()),
            };
            let hr = ctx
                .ctx
                .CreateSolidColorBrush(&mk_color_f(color), &brush_props, &mut brush);
            assert!(SUCCEEDED(hr));
            Brush(BrushImpl::SolidColor(ComPtr::from_raw(brush)))
        }
    }

    pub fn new_linear_gradient(
        ctx: &DrawContext,
        stops: &GradientStopCollection,
        start: Point,
        end: Point,
        opacity: f64,
    ) -> Brush {
        unsafe {
            let mut brush = ptr::null_mut();
            let brush_props = D2D1_BRUSH_PROPERTIES {
                opacity: opacity as f32, // FIXME what's the difference with color.a?
                transform: mk_matrix_3x2(&Transform::identity()),
            };
            let linear_gradient_props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: mk_point_f(start),
                endPoint: mk_point_f(end),
            };
            let hr = ctx.ctx.CreateLinearGradientBrush(
                &linear_gradient_props,
                &brush_props,
                stops.0.as_raw().cast(),
                &mut brush,
            );
            assert!(SUCCEEDED(hr));
            Brush(BrushImpl::LinearGradient(ComPtr::from_raw(brush)))
        }
    }

    pub fn as_raw_brush(&self) -> *mut ID2D1Brush {
        match &self.0 {
            BrushImpl::SolidColor(ptr) => ptr.as_raw().cast(),
            BrushImpl::RadialGradient(ptr) => ptr.as_raw().cast(),
            BrushImpl::LinearGradient(ptr) => ptr.as_raw().cast(),
        }
    }
}

/// Trait for objects that can be converted into a brush.
pub trait IntoBrush {
    fn into_brush(self, target: &DrawContext) -> Brush;
}

impl IntoBrush for LinSrgba<f32> {
    fn into_brush(self, ctx: &DrawContext) -> Brush {
        Brush::new_solid_color(ctx, Color::from_linear(self))
    }
}

impl IntoBrush for LinSrgb<f32> {
    fn into_brush(self, ctx: &DrawContext) -> Brush {
        Brush::new_solid_color(
            ctx,
            Alpha {
                color: Srgb::from_linear(self),
                alpha: 1.0,
            },
        )
    }
}

impl IntoBrush for Brush {
    fn into_brush(self, _ctx: &DrawContext) -> Brush {
        self
    }
}
