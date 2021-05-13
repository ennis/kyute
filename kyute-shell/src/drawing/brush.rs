//! Brushes.
use crate::{
    bindings::Windows::Win32::Direct2D::{
        ID2D1Brush, ID2D1LinearGradientBrush, ID2D1RadialGradientBrush, ID2D1SolidColorBrush,
        D2D1_BRUSH_PROPERTIES, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
    },
    drawing::{
        context::DrawContext, mk_color_f, mk_matrix_3x2, mk_point_f, Color, GradientStopCollection,
        Point, Transform,
    },
};
use palette::{Alpha, LinSrgb, LinSrgba, Srgb};
use windows::Interface;

pub(crate) enum BrushImpl {
    SolidColor(ID2D1SolidColorBrush),
    RadialGradient(ID2D1RadialGradientBrush),
    LinearGradient(ID2D1LinearGradientBrush),
}

/// Brushes to fill or stroke geometry.
pub struct Brush(pub(crate) BrushImpl);

impl Brush {
    pub fn new_solid_color(ctx: &DrawContext, color: Color) -> Brush {
        unsafe {
            let brush_props = D2D1_BRUSH_PROPERTIES {
                opacity: 1.0, // FIXME what's the difference with color.a?
                transform: mk_matrix_3x2(&Transform::identity()),
            };

            let mut brush = None;
            let brush = ctx
                .ctx
                .CreateSolidColorBrush(&mk_color_f(color), &brush_props, &mut brush)
                .and_some(brush)
                .unwrap();

            Brush(BrushImpl::SolidColor(brush))
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
            let brush_props = D2D1_BRUSH_PROPERTIES {
                opacity: opacity as f32, // FIXME what's the difference with color.a?
                transform: mk_matrix_3x2(&Transform::identity()),
            };
            let linear_gradient_props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: mk_point_f(start),
                endPoint: mk_point_f(end),
            };
            let mut brush = None;
            let brush = ctx
                .ctx
                .CreateLinearGradientBrush(
                    &linear_gradient_props,
                    &brush_props,
                    &stops.0,
                    &mut brush,
                )
                .and_some(brush)
                .unwrap();
            Brush(BrushImpl::LinearGradient(brush))
        }
    }

    pub(crate) fn to_base_brush(&self) -> ID2D1Brush {
        match &self.0 {
            BrushImpl::SolidColor(b) => b.cast().unwrap(),
            BrushImpl::RadialGradient(b) => b.cast().unwrap(),
            BrushImpl::LinearGradient(b) => b.cast().unwrap(),
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
